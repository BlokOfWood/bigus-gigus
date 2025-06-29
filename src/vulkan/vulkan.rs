use std::{ffi::CStr, sync::Arc};

use ash::{
    vk::{self, ApplicationInfo, KHR_SWAPCHAIN_NAME},
    Entry, Instance,
};
use winit::{
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use crate::vulkan::{
    command_pool::{create_command_pool, create_render_pass},
    device_and_queues::QueueFamilyIndices,
    image::{create_depth_resources, create_image_views},
    pipeline::{create_descriptor_set_layout, create_graphics_pipeline},
    swap_chain::{create_frame_buffers, create_swap_chain},
    window::{create_surface, enumerate_required_extensions},
};

use crate::vulkan::device_and_queues::BigusDevice;

/*use super::{
    buffers::{create_index_buffer, create_uniform_buffers, create_vertex_buffer},
    command_pool::{create_render_pass, CommandPool},
    device_and_queues::QueueFamilies,
    image::{
        create_image_views, create_texture_image, create_texture_image_view, create_texture_sampler,
    },
    pipeline::{create_descriptor_set_layout, create_descriptor_sets, create_graphics_pipeline},
    swap_chain::{create_frame_buffers, create_swap_chain},
    ubo::UniformBufferObject,
};*/

const ENGINE_NAME: &str = "Very cool engine";
const APPLICATION_NAME: &str = "Very cool application";
pub(super) const REQUIRED_EXTENSIONS: [&CStr; 1] = [KHR_SWAPCHAIN_NAME];

pub struct VulkanRenderer {
    pub(super) inst: Instance,
    /*pub(super) device: Arc<BigusDevice>,
    queues: QueueFamilies,
    pub(super) surface: Arc<Surface>,
    pub(super) swap_chain: Arc<Swapchain>,
    pub(super) images: Vec<Arc<Image>>,
    pub(super) image_format: Format,
    pub(super) image_extent: [u32; 2],
    pub(super) image_views: Vec<Arc<ImageView>>,
    pub(super) depth_image_view: Arc<ImageView>,
    pub(super) render_pass: Arc<RenderPass>,
    graphics_pipeline: Arc<Pipeline>,
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    pub(super) uniform_buffers: Vec<Buffer>,
    descriptor_sets: Vec<Arc<DescriptorSet>>,
    pub(super) frame_buffers: Vec<Arc<Framebuffer>>,
    command_pool: CommandPool,
    fences: Vec<Option<Arc<Fence>>>,
    pub(super) start_time: Instant,*/
}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>, event_loop: &ActiveEventLoop) -> Self {
        let entry = Entry::linked();

        let api_version = match unsafe { entry.try_enumerate_instance_version() } {
            Ok(version) => match version {
                Some(version) => format!(
                    "{}.{}.{}",
                    vk::api_version_major(version),
                    vk::api_version_minor(version),
                    vk::api_version_patch(version)
                ),
                None => "1.0.x".to_string(),
            },
            Err(err) => {
                println!("Failed to acquire api version. Error: {}", err);
                "?".to_string()
            }
        };

        let application_info = ApplicationInfo {
            p_application_name: APPLICATION_NAME.as_ptr() as *const i8,
            p_engine_name: ENGINE_NAME.as_ptr() as *const i8,
            ..Default::default()
        };

        let window_required_extensions =
            enumerate_required_extensions(event_loop.display_handle().unwrap().into())
                .expect("Failed to enumerate required extensions.");

        let mut enabled_layers = Vec::new();
        let mut enabled_extensions = Vec::new();
        window_required_extensions
            .iter()
            .for_each(|extension| enabled_extensions.push(*extension));

        if cfg!(debug_assertions) {
            println!("Enabling validation layers");
            enabled_layers.push("VK_LAYER_KHRONOS_validation".as_ptr() as *const i8);
        }

        let create_info = vk::InstanceCreateInfo {
            enabled_layer_count: enabled_layers.len() as u32,
            pp_enabled_layer_names: enabled_layers.as_ptr(),
            enabled_extension_count: enabled_extensions.len() as u32,
            pp_enabled_extension_names: enabled_extensions.as_ptr(),
            p_application_info: &application_info,
            ..Default::default()
        };

        let inst = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Vulkan unavailable")
        };
        println!(
            "Loaded Vulkan with version: {}. Extension count: {}",
            api_version,
            unsafe { entry.enumerate_instance_extension_properties(None) }
                .into_iter()
                .count()
        );

        let surface = unsafe {
            create_surface(
                &entry,
                &inst,
                window.display_handle().unwrap().into(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .expect("Failed to create surface from window.")
        };

        let bigus_device = BigusDevice::new(&entry, &inst, surface);

        let queue_family_indices =
            QueueFamilyIndices::find_queue_families(&entry, &inst, bigus_device.phys_dev, surface);

        let (swap_chain, images, image_format, image_extent) = create_swap_chain(
            ash::khr::swapchain::Device::new(&inst, &bigus_device.dev),
            bigus_device.phys_dev,
            ash::khr::surface::Instance::new(&entry, &inst),
            surface,
            [window.inner_size().width, window.inner_size().height],
            &queue_family_indices,
        );

        let image_views = create_image_views(&bigus_device, images, image_format.format);

        let (depth_image_view, depth_image_memory, depth_image_format) = create_depth_resources(
            &inst,
            &bigus_device,
            [image_extent.width, image_extent.height],
        );

        let render_pass =
            create_render_pass(&bigus_device, image_format.format, depth_image_format);

        let descriptor_set_layout = create_descriptor_set_layout(&bigus_device);

        let graphics_pipeline =
            create_graphics_pipeline(&bigus_device, render_pass, vec![descriptor_set_layout]);

        let command_pool = create_command_pool(&bigus_device, &queue_family_indices);

        let depth_resources = create_depth_resources(
            &inst,
            &bigus_device,
            [image_extent.width, image_extent.height],
        );

        let frame_buffers = create_frame_buffers(
            &bigus_device,
            render_pass.clone(),
            image_views,
            depth_image_view.clone(),
            image_extent,
        );
        /*
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
        );*/

        VulkanRenderer {
            inst,
            /*    fences: vec![None; (&images).len()],
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
            start_time: Instant::now(),*/
        }
    }

    /*pub fn draw_frame(&mut self) {
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
    }*/
}
