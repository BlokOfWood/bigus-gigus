use std::{sync::Arc, time::Instant};

use ash::vk::Fence;
use vulkano::{
    buffer::{Buffer, Subbuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        CommandBufferExecFuture,
    },
    descriptor_set::PersistentDescriptorSet,
    format::Format,
    image::{view::ImageView, Image},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{suballocator, GenericMemoryAllocator, GenericMemoryAllocatorCreateInfo},
    pipeline::GraphicsPipeline,
    render_pass::{Framebuffer, RenderPass},
    swapchain::{
        acquire_next_image, PresentFuture, Surface, SurfaceInfo, Swapchain, SwapchainAcquireFuture,
        SwapchainPresentInfo,
    },
    sync::{
        self,
        future::{FenceSignalFuture, JoinFuture},
        GpuFuture,
    },
    Validated, VulkanError, VulkanLibrary,
};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::vulkan::{device_and_queues::BigusDevice, image::create_depth_resources};

use super::{
    buffers::{create_index_buffer, create_uniform_buffers, create_vertex_buffer},
    command_pool::{create_render_pass, CommandPool},
    device_and_queues::QueueFamilies,
    image::{
        create_image_views, create_texture_image, create_texture_image_view, create_texture_sampler,
    },
    pipeline::{create_descriptor_set_layout, create_descriptor_sets, create_graphics_pipeline},
    swap_chain::{create_frame_buffers, create_swap_chain},
    ubo::UniformBufferObject,
};

const ENGINE_NAME: &str = "Very cool engine";
const APPLICATION_NAME: &str = "Very cool application";

pub struct VulkanRenderer {
    pub(super) inst: Arc<Instance>,
    pub(super) device: Arc<BigusDevice>,
    queues: QueueFamilies,
    pub(super) surface: Arc<Surface>,
    pub(super) swap_chain: Arc<Swapchain>,
    pub(super) images: Vec<Arc<Image>>,
    pub(super) image_format: Format,
    pub(super) image_extent: [u32; 2],
    pub(super) image_views: Vec<Arc<ImageView>>,
    pub(super) depth_image_view: Arc<ImageView>,
    pub(super) render_pass: Arc<RenderPass>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    pub(super) uniform_buffers: Vec<Subbuffer<UniformBufferObject>>,
    descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,
    pub(super) frame_buffers: Vec<Arc<Framebuffer>>,
    command_pool: CommandPool,
    fences: Vec<
        Option<
            Arc<
                FenceSignalFuture<
                    PresentFuture<
                        CommandBufferExecFuture<
                            JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>,
                        >,
                    >,
                >,
            >,
        >,
    >,
    pub(super) start_time: Instant,
}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>, event_loop: &ActiveEventLoop) -> Self {
        let vk_library = VulkanLibrary::new().expect("Vulkan unavailable");
        println!(
            "Loaded Vulkan with version: {}. Extension count: {}",
            vk_library.api_version(),
            vk_library.supported_extensions().into_iter().count()
        );
        let mut enabled_extensions = Surface::required_extensions(event_loop);
        let mut enabled_layers = Vec::new();
        if cfg!(debug_assertions) {
            println!("Enabling validation layers");
            enabled_extensions.ext_debug_utils = true;
            enabled_layers.push("VK_LAYER_KHRONOS_validation".to_string());
        }
        let instance_info = InstanceCreateInfo {
            application_name: Some(APPLICATION_NAME.to_string()),
            engine_name: Some(ENGINE_NAME.to_string()),
            enabled_extensions,
            enabled_layers,
            ..Default::default()
        };

        let inst =
            Instance::new(vk_library, instance_info).expect("Failed to create Vulkan instance");

        let surface = Surface::from_window(inst.clone(), window.clone())
            .expect("Failed to create surface from window.");

        let device = BigusDevice::new(inst.clone(), surface.clone());

        let allocator: Arc<GenericMemoryAllocator<suballocator::FreeListAllocator>> =
            Arc::new(GenericMemoryAllocator::new(
                device.device(),
                GenericMemoryAllocatorCreateInfo {
                    block_sizes: device
                        .phys_device()
                        .memory_properties()
                        .memory_types
                        .iter()
                        .map(|_| 0xFF)
                        .collect::<Vec<u64>>()
                        .as_slice(),
                    ..Default::default()
                },
            ));

        let (swap_chain, images, image_format, image_extent) = create_swap_chain(
            device.device(),
            surface.clone(),
            device
                .phys_device()
                .clone()
                .surface_capabilities(Arc::as_ref(&surface), SurfaceInfo::default())
                .unwrap(),
            window,
        );

        let image_views = create_image_views(&images, image_format);

        let (depth_image_view, depth_image_format) = create_depth_resources(
            device.clone(),
            allocator.clone(),
            [image_extent[0], image_extent[1], 1],
        );

        let render_pass = create_render_pass(device.device(), image_format, depth_image_format);

        let descriptor_set_layout = create_descriptor_set_layout(device.device());

        let (_, graphics_pipeline) = create_graphics_pipeline(
            device.device(),
            render_pass.clone(),
            image_extent,
            vec![descriptor_set_layout.clone()],
        );

        let frame_buffers = create_frame_buffers(
            render_pass.clone(),
            &image_views,
            image_extent,
            depth_image_view.clone(),
        );

        let command_pool = CommandPool::new(device.clone(), surface.clone());

        let queues = device.queues();

        let vertex_buffer = create_vertex_buffer(
            device.device(),
            queues.graphics_queue.clone(),
            &command_pool,
            allocator.clone(),
        );

        let index_buffer = create_index_buffer(
            device.device(),
            queues.graphics_queue.clone(),
            &command_pool,
            allocator.clone(),
        );

        let uniform_buffers = create_uniform_buffers((&images).len(), allocator.clone()).unwrap();
        let image = create_texture_image(
            allocator.clone(),
            &StandardCommandBufferAllocator::new(
                device.device(),
                StandardCommandBufferAllocatorCreateInfo {
                    primary_buffer_count: 1,
                    ..Default::default()
                },
            ),
            queues.graphics_queue.clone(),
            device.device(),
        );

        let image_view = create_texture_image_view(image.clone());

        let texture_sampler = create_texture_sampler(device.device());

        let descriptor_sets = create_descriptor_sets(
            descriptor_set_layout.clone(),
            &uniform_buffers,
            device.device(),
            image_view.clone(),
            texture_sampler.clone(),
        );

        VulkanRenderer {
            fences: vec![None; (&images).len()],
            inst,
            device: Arc::new(device),
            queues,
            surface,
            swap_chain,
            images,
            image_format,
            image_extent,
            image_views,
            depth_image_view,
            render_pass,
            vertex_buffer,
            index_buffer,
            descriptor_sets,
            uniform_buffers,
            graphics_pipeline,
            frame_buffers,
            command_pool,
            start_time: Instant::now(),
        }
    }

    pub fn draw_frame(&mut self) {
        // Acquires an image from the swap chain to draw unto.
        // The swap chain is basically a buffer of images, where one of them is being displayed while we draw unto the other one.
        // Basically decouples presenting an image from drawing the image, so that we can sync with the monitor's refresh rate.
        let (image_idx, _is_suboptimal, acquire_future) =
            match acquire_next_image(self.swap_chain.clone(), None).map_err(Validated::unwrap) {
                Ok(acquired_image_details) => acquired_image_details,
                Err(VulkanError::OutOfDate) => {
                    return;
                }
                Err(err) => {
                    println!("Failed to acquire new image. Error: {}", err);
                    return;
                }
            };

        if let Some(image_fence) = &self.fences[image_idx as usize] {
            image_fence.wait(None).unwrap();
        }

        self.update_uniform_buffer(
            image_idx.try_into().unwrap(),
            (self.image_extent[0] / self.image_extent[1]) as f32,
        );

        let command_buffer = self.command_pool.record_render_pass(
            self.render_pass.clone(),
            self.frame_buffers[image_idx as usize].clone(),
            self.graphics_pipeline.clone(),
            self.descriptor_sets[image_idx as usize].clone(),
            self.image_extent,
            self.vertex_buffer.clone(),
            self.index_buffer.clone(),
        );

        let previous_future = match self.fences[image_idx as usize].clone() {
            None => {
                let mut now = sync::now(self.device.device());
                now.cleanup_finished();
                now.boxed()
            }
            Some(fence) => fence.boxed(),
        };

        let command_execution_result = previous_future
            .join(acquire_future)
            .then_execute(self.queues.graphics_queue.clone(), command_buffer);

        let presentation_result = match command_execution_result {
            Ok(future) => future
                .then_swapchain_present(
                    self.queues.graphics_queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(self.swap_chain.clone(), image_idx),
                )
                .then_signal_fence_and_flush(),
            Err(err) => {
                print!("Failed to execute command buffer {}", err.to_string());
                return;
            }
        };

        match presentation_result {
            Ok(new_fence) => self.fences[image_idx as usize] = Some(Arc::new(new_fence)),
            Err(err) => match err {
                Validated::Error(err) => match err {
                    VulkanError::OutOfDate => {
                        return;
                    }
                    err => {
                        println!("Failed to present image, Error: {}", err);
                        return;
                    }
                },
                Validated::ValidationError(err) => println!("wee {}", err),
            },
        };
    }
}
