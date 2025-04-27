use std::{collections::BTreeMap, mem, num::NonZero, slice, sync::Arc, time::Instant, vec};

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::CommandBufferExecFuture,
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo,
    },
    format::Format,
    image::{
        sampler::ComponentMapping,
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageAspects, ImageLayout, ImageSubresourceRange, ImageUsage, SampleCount,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::{
        allocator::{
            suballocator, AllocationCreateInfo, DeviceLayout, GenericMemoryAllocator,
            GenericMemoryAllocatorCreateInfo, MemoryAllocator, MemoryTypeFilter,
        },
        DeviceAlignment, MemoryPropertyFlags,
    },
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState, ColorComponents},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
            subpass::PipelineSubpassType,
            vertex_input::VertexInputState,
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineCreateFlags, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{
        AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
        Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreateInfo, Subpass,
        SubpassDependency, SubpassDescription,
    },
    shader::{EntryPoint, ShaderStages},
    swapchain::{
        acquire_next_image, CompositeAlpha, PresentFuture, Surface, SurfaceCapabilities,
        SurfaceInfo, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{
        self,
        future::{FenceSignalFuture, JoinFuture},
        AccessFlags, GpuFuture, PipelineStages, Sharing,
    },
    Validated, VulkanError, VulkanLibrary,
};
use winit::{
    event_loop::ActiveEventLoop, raw_window_handle_05::HasRawDisplayHandle, window::Window,
};

use crate::{
    math::{
        graphics_ops::{look_at, perspective},
        matrix::Matrix4,
        quaternion::Quaternion,
        vector::{Vector3, VECTOR3_ZERO},
    },
    vulkan::queue_family::QueueFamilyIndices,
};

use super::{
    command_pool::CommandPool,
    queue_family::QueueFamilies,
    shader::Shaders,
    swap_chain::SwapChainSupport,
    ubo::UniformBufferObject,
    vertex_buffer::{Vertex, VertexBuffer, VertexData, INDICES, VERTICES},
};

const ENGINE_NAME: &str = "Very cool engine";
const APPLICATION_NAME: &str = "Very cool application";

pub struct VulkanRenderer {
    inst: Arc<Instance>,
    phys_dev: Arc<PhysicalDevice>,
    dev: Arc<Device>,
    queues: QueueFamilies,
    surface: Arc<Surface>,
    swap_chain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    image_format: Format,
    image_extent: [u32; 2],
    image_views: Vec<Arc<ImageView>>,
    render_pass: Arc<RenderPass>,
    descriptor_set_layout: Arc<DescriptorSetLayout>,

    graphics_pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    uniform_buffers: Vec<Subbuffer<UniformBufferObject>>,
    descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,
    frame_buffers: Vec<Arc<Framebuffer>>,
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
    fence_idx: u32,
    start_time: Instant,
}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>, event_loop: &ActiveEventLoop) -> Self {
        let inst = Self::create_vulkan_instance(event_loop);
        let surface = Surface::from_window(inst.clone(), window.clone())
            .expect("Failed to create surface from window.");
        let phys_dev = Self::create_vulkan_physical_device(inst.clone(), surface.clone());

        let (dev, queues) = Self::create_device_and_queues(phys_dev.clone(), surface.clone());

        let (swap_chain, images, format, extent) = Self::create_swap_chain(
            phys_dev.clone(),
            dev.clone(),
            surface.clone(),
            phys_dev
                .clone()
                .surface_capabilities(Arc::as_ref(&surface), SurfaceInfo::default())
                .unwrap(),
            window,
        );

        let image_views = Self::create_image_views(&images, format);

        let render_pass = Self::create_render_pass(dev.clone(), format);

        let descriptor_set_layout = match Self::create_descriptor_set_layout(dev.clone()) {
            Ok(descriptor_set_layout) => descriptor_set_layout,
            Err(err) => panic!("Failed to create descriptor set layout! Error: {}", err),
        };

        let (pipeline_layout, graphics_pipeline) = Self::create_graphics_pipeline(
            dev.clone(),
            render_pass.clone(),
            extent,
            vec![descriptor_set_layout.clone()],
        );

        let frame_buffers = Self::create_frame_buffers(render_pass.clone(), &image_views, extent);

        let command_pool = CommandPool::new(dev.clone(), phys_dev.clone(), surface.clone());

        let memory_allocator = Arc::new(Self::create_memory_allocator(dev.clone()));

        let vertex_buffer = Self::create_vertex_buffer(
            dev.clone(),
            queues.graphics_queue.clone(),
            &command_pool,
            memory_allocator.clone(),
        );

        let index_buffer = Self::create_index_buffer(
            dev.clone(),
            queues.graphics_queue.clone(),
            &command_pool,
            memory_allocator.clone(),
        );

        let uniform_buffers =
            Self::create_uniform_buffers((&images).len(), memory_allocator.clone()).unwrap();
        let descriptor_sets = Self::create_descriptor_sets(
            descriptor_set_layout.clone(),
            &uniform_buffers,
            dev.clone(),
        );

        VulkanRenderer {
            fences: vec![None; (&images).len()],
            inst,
            phys_dev,
            dev: dev.clone(),
            queues,
            surface,
            swap_chain,
            images,
            image_format: format,
            image_extent: extent,
            image_views,
            render_pass,
            vertex_buffer,
            index_buffer,
            descriptor_set_layout,
            descriptor_sets,
            uniform_buffers,
            graphics_pipeline,
            frame_buffers,
            command_pool,
            fence_idx: 0,
            start_time: Instant::now(),
        }
    }

    pub fn recreate_swap_chain(&mut self, window: Arc<Window>) {
        let surface = Surface::from_window(self.inst.clone(), window.clone())
            .expect("Failed to create surface from window.");

        self.surface = surface.clone();

        let swap_extent = Self::choose_swap_extent(
            self.phys_dev
                .surface_capabilities(&surface, SurfaceInfo::default())
                .unwrap(),
            window.clone(),
        );

        let (swap_chain, images) = self
            .swap_chain
            .recreate(SwapchainCreateInfo {
                image_extent: swap_extent,
                ..self.swap_chain.create_info()
            })
            .expect("Failed to recreate swap chain on resize!");

        self.swap_chain = swap_chain;
        self.images = images;
        self.image_extent = swap_extent;

        self.image_views = VulkanRenderer::create_image_views(&self.images, self.image_format);

        self.frame_buffers = VulkanRenderer::create_frame_buffers(
            self.render_pass.clone(),
            &self.image_views,
            self.image_extent,
        );
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
                let mut now = sync::now(self.dev.clone());
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

        let new_fence = match presentation_result.map_err(Validated::unwrap) {
            Ok(new_fence) => new_fence,
            Err(VulkanError::OutOfDate) => {
                return;
            }
            Err(err) => {
                println!("Failed to present image, Error: {}", err);
                return;
            }
        };

        self.fences[image_idx as usize] = Some(Arc::new(new_fence));
        self.fence_idx = image_idx;
    }

    fn create_vulkan_instance(event_loop: &impl HasRawDisplayHandle) -> Arc<Instance> {
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

        Instance::new(vk_library, instance_info).expect("Failed to create Vulkan instance")
    }

    fn create_vulkan_physical_device(
        vk_instance: Arc<Instance>,
        vk_surface: Arc<Surface>,
    ) -> Arc<PhysicalDevice> {
        let vk_phys_dev = vk_instance
            .enumerate_physical_devices()
            .unwrap()
            .find(|device| Self::is_device_suitable(device, vk_surface.clone()))
            .expect("No physical devices found");

        println!("Using device: {}", vk_phys_dev.properties().device_name);

        vk_phys_dev
    }

    fn create_device_and_queues(
        vk_phys_dev: Arc<PhysicalDevice>,
        vk_surface: Arc<Surface>,
    ) -> (Arc<Device>, QueueFamilies) {
        let queue_family_indicies =
            QueueFamilyIndices::find_queue_families(vk_phys_dev.clone(), vk_surface.clone());

        let graphics_queue_create_info = QueueCreateInfo {
            queue_family_index: queue_family_indicies.graphics_family.unwrap(),
            queues: vec![1f32],
            ..Default::default()
        };
        let presentation_queue_create_info = QueueCreateInfo {
            queue_family_index: queue_family_indicies.presentation_family.unwrap(),
            queues: vec![1f32],
            ..Default::default()
        };

        let enabled_features = Features::default();

        let mut enabled_extensions = DeviceExtensions::default();
        enabled_extensions.khr_swapchain = true;

        let device_create_info = DeviceCreateInfo {
            queue_create_infos: vec![graphics_queue_create_info, presentation_queue_create_info],
            enabled_features,
            enabled_extensions,
            ..Default::default()
        };
        let (vk_dev, mut vk_queue) = Device::new(vk_phys_dev.clone(), device_create_info)
            .expect("Failed to create logical device for phyisical device.");

        let graphics_queue = vk_queue
            .find(|queue| {
                queue.queue_family_index() == queue_family_indicies.graphics_family.unwrap()
            })
            .unwrap();
        let presentation_queue = vk_queue
            .find(|queue| {
                queue.queue_family_index() == queue_family_indicies.presentation_family.unwrap()
            })
            .unwrap();

        (
            vk_dev,
            QueueFamilies {
                graphics_queue,
                _presentation_queue: presentation_queue,
            },
        )
    }

    fn create_swap_chain(
        phys_dev: Arc<PhysicalDevice>,
        dev: Arc<Device>,
        surface: Arc<Surface>,
        capabilities: SurfaceCapabilities,
        window: Arc<Window>,
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>, Format, [u32; 2]) {
        let swap_chain_support = SwapChainSupport::new(phys_dev.clone(), surface.clone());

        let surface_format = swap_chain_support.choose_surface_format();
        let present_mode = swap_chain_support.choose_present_mode();
        let swap_extent = Self::choose_swap_extent(capabilities, window.clone());

        let max_image_count = swap_chain_support.capabilities.max_image_count.unwrap();
        let mut image_count = swap_chain_support.capabilities.min_image_count + 1;

        if max_image_count > 0 && image_count > max_image_count {
            image_count = max_image_count;
        };

        let query_family_indicies =
            QueueFamilyIndices::find_queue_families(phys_dev.clone(), surface.clone());
        let sharing_mode = if query_family_indicies.graphics_family.unwrap()
            == query_family_indicies.presentation_family.unwrap()
        {
            Sharing::Concurrent(
                vec![
                    query_family_indicies.graphics_family.unwrap(),
                    query_family_indicies.presentation_family.unwrap(),
                ]
                .into(),
            )
        } else {
            Sharing::Exclusive
        };

        let create_info = SwapchainCreateInfo {
            min_image_count: image_count,
            image_format: surface_format.0,
            image_color_space: surface_format.1,
            image_extent: swap_extent,
            image_array_layers: 1,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            image_sharing: sharing_mode,
            pre_transform: swap_chain_support.capabilities.current_transform,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode,
            clipped: true,
            ..Default::default()
        };

        let create_results = Swapchain::new(dev, surface.clone(), create_info).unwrap();

        (
            create_results.0,
            create_results.1,
            surface_format.0,
            swap_extent,
        )
    }

    fn is_device_suitable(device: &Arc<PhysicalDevice>, surface: Arc<Surface>) -> bool {
        let queue_families =
            QueueFamilyIndices::find_queue_families(device.clone(), surface.clone());
        let swap_chain_support = SwapChainSupport::new(device.clone(), surface.clone());

        return queue_families.has_required_families()
            && device.supported_extensions().khr_swapchain
            && !swap_chain_support.formats.is_empty()
            && !swap_chain_support.present_modes.is_empty();
    }

    fn choose_swap_extent(capabilities: SurfaceCapabilities, window: Arc<Window>) -> [u32; 2] {
        let current_extent = capabilities.current_extent.unwrap();
        if current_extent[0] != u32::MAX {
            return capabilities.current_extent.unwrap();
        } else {
            let inner_size = window.inner_size();
            let width = inner_size.width.clamp(
                capabilities.min_image_extent[0],
                capabilities.max_image_extent[0],
            );
            let height = inner_size.height.clamp(
                capabilities.min_image_extent[1],
                capabilities.max_image_extent[1],
            );

            [width, height]
        }
    }

    fn create_image_views(
        swap_chain_images: &Vec<Arc<Image>>,
        image_format: Format,
    ) -> Vec<Arc<ImageView>> {
        let mut image_views: Vec<Arc<ImageView>> = Vec::with_capacity(swap_chain_images.len());

        for image in swap_chain_images {
            let create_info = ImageViewCreateInfo {
                view_type: ImageViewType::Dim2d,
                format: image_format,
                component_mapping: ComponentMapping::identity(),
                subresource_range: ImageSubresourceRange {
                    array_layers: 0..1,
                    aspects: ImageAspects::COLOR,
                    mip_levels: 0..1,
                },
                ..Default::default()
            };

            let image_view = ImageView::new(image.clone(), create_info);
            image_views.push(image_view.unwrap());
        }

        image_views
    }

    fn create_render_pass(dev: Arc<Device>, image_format: Format) -> Arc<RenderPass> {
        let color_attachment = AttachmentDescription {
            format: image_format,
            samples: SampleCount::Sample1,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::Store,
            stencil_load_op: Some(AttachmentLoadOp::DontCare),
            stencil_store_op: Some(AttachmentStoreOp::DontCare),
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::PresentSrc,
            ..Default::default()
        };

        let subpass_dependency = SubpassDependency {
            src_subpass: None,
            dst_subpass: Some(0),
            src_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
            src_access: AccessFlags::empty(),
            dst_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
            dst_access: AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let color_attachment_ref = AttachmentReference {
            attachment: 0,
            layout: ImageLayout::ColorAttachmentOptimal,
            ..Default::default()
        };

        let subpass_desc = SubpassDescription {
            color_attachments: vec![Some(color_attachment_ref)],
            ..Default::default()
        };

        let render_pass_create_info = RenderPassCreateInfo {
            attachments: vec![color_attachment],
            subpasses: vec![subpass_desc],
            dependencies: vec![subpass_dependency],
            ..Default::default()
        };

        RenderPass::new(dev, render_pass_create_info).unwrap()
    }

    fn create_graphics_pipeline(
        dev: Arc<Device>,
        render_pass: Arc<RenderPass>,
        image_extent: [u32; 2],
        descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
    ) -> (Arc<PipelineLayout>, Arc<GraphicsPipeline>) {
        let shaders = Shaders::new(dev.clone(), "src/shaders/vert.spv", "src/shaders/frag.spv");

        let vert_entry_point: EntryPoint = shaders.vert_shader.single_entry_point().unwrap();
        let vert_stage_info = PipelineShaderStageCreateInfo::new(vert_entry_point);
        let frag_entry_point: EntryPoint = shaders.frag_shader.single_entry_point().unwrap();
        let frag_stage_info = PipelineShaderStageCreateInfo::new(frag_entry_point);

        let vertex_input_state = VertexInputState::new()
            .binding(0, VertexBuffer::get_binding_description())
            .attributes(VertexBuffer::get_attribute_descriptions());

        let input_assembly_state = InputAssemblyState {
            topology: PrimitiveTopology::TriangleList,
            primitive_restart_enable: false,
            ..Default::default()
        };
        let viewport = Viewport {
            offset: [0f32, 0f32],
            extent: [image_extent[0] as f32, image_extent[1] as f32],
            depth_range: 0f32..=1f32,
        };
        let scissor = Scissor {
            offset: [0, 0],
            extent: image_extent,
        };
        let viewport_state = ViewportState {
            viewports: [viewport].into(),
            scissors: [scissor].into(),
            ..Default::default()
        };

        let rasterization_state = RasterizationState {
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: PolygonMode::Fill,
            line_width: 1.0f32,
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            depth_bias: None,
            ..Default::default()
        };

        let multisample_state = MultisampleState {
            sample_shading: None,
            rasterization_samples: SampleCount::Sample1,
            ..Default::default()
        };

        let color_blend_attachment = ColorBlendAttachmentState {
            color_write_enable: true,
            color_write_mask: ColorComponents::all(),
            blend: None,
            ..Default::default()
        };

        let color_blending = ColorBlendState {
            logic_op: None,
            attachments: vec![color_blend_attachment],
            ..Default::default()
        };

        let pipeline_layout_info = PipelineLayoutCreateInfo {
            set_layouts: descriptor_set_layouts,
            ..Default::default()
        };

        let pipeline_layout = PipelineLayout::new(dev.clone(), pipeline_layout_info).unwrap();

        let subpass = Subpass::from(render_pass, 0).unwrap();

        let mut graphics_pipeline_create_info =
            GraphicsPipelineCreateInfo::layout(pipeline_layout.clone());
        graphics_pipeline_create_info.stages = vec![vert_stage_info, frag_stage_info].into();
        graphics_pipeline_create_info.vertex_input_state = Some(vertex_input_state);
        graphics_pipeline_create_info.input_assembly_state = Some(input_assembly_state);
        graphics_pipeline_create_info.viewport_state = Some(viewport_state);
        graphics_pipeline_create_info.rasterization_state = Some(rasterization_state);
        graphics_pipeline_create_info.multisample_state = Some(multisample_state);
        graphics_pipeline_create_info.depth_stencil_state = None;
        graphics_pipeline_create_info.color_blend_state = Some(color_blending);
        graphics_pipeline_create_info.dynamic_state =
            ahash::HashSet::from_iter([DynamicState::Viewport, DynamicState::Scissor]);
        graphics_pipeline_create_info.subpass = Some(PipelineSubpassType::BeginRenderPass(subpass));
        graphics_pipeline_create_info.base_pipeline = None;
        graphics_pipeline_create_info.tessellation_state = None;
        graphics_pipeline_create_info.discard_rectangle_state = None;
        graphics_pipeline_create_info.flags = PipelineCreateFlags::empty();

        (
            pipeline_layout,
            GraphicsPipeline::new(dev.clone(), None, graphics_pipeline_create_info).unwrap(),
        )
    }

    fn create_frame_buffers(
        render_pass: Arc<RenderPass>,
        image_views: &Vec<Arc<ImageView>>,
        image_extent: [u32; 2],
    ) -> Vec<Arc<Framebuffer>> {
        image_views
            .iter()
            .map(|image_view| {
                let framebuffer_create_info = FramebufferCreateInfo {
                    attachments: vec![image_view.clone()],
                    extent: image_extent,
                    layers: 1,
                    ..Default::default()
                };

                Framebuffer::new(render_pass.clone(), framebuffer_create_info).unwrap()
            })
            .collect()
    }

    fn create_memory_allocator(
        device: Arc<Device>,
    ) -> GenericMemoryAllocator<suballocator::FreeListAllocator> {
        let max_alloc_size = device
            .physical_device()
            .properties()
            .max_memory_allocation_size
            .unwrap_or(0xFF);
        let block_sizes: Vec<u64> = device
            .physical_device()
            .memory_properties()
            .memory_types
            .iter()
            .map(|_| 0xFF)
            .collect();

        GenericMemoryAllocator::new(
            device,
            GenericMemoryAllocatorCreateInfo {
                block_sizes: block_sizes.as_slice(),
                ..Default::default()
            },
        )
    }

    fn create_vertex_buffer(
        dev: Arc<Device>,
        graphics_queue: Arc<Queue>,
        command_pool: &CommandPool,
        allocator: Arc<dyn MemoryAllocator>,
    ) -> Arc<Buffer> {
        let staging_buffer = Buffer::from_data(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                sharing: Sharing::Exclusive,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter {
                    required_flags: MemoryPropertyFlags::HOST_VISIBLE
                        | MemoryPropertyFlags::HOST_COHERENT,
                    ..Default::default()
                },
                ..Default::default()
            },
            VertexData { vertices: VERTICES },
        )
        .unwrap();

        let vertex_buffer = Buffer::new(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX_BUFFER,
                sharing: Sharing::Exclusive,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter {
                    required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
                    ..Default::default()
                },
                ..Default::default()
            },
            DeviceLayout::from_size_alignment(
                (size_of::<VertexData>()) as u64,
                DeviceAlignment::MIN.into(),
            )
            .unwrap(),
        )
        .unwrap();

        let command_buffer = command_pool
            .record_copy_pass(staging_buffer.into_bytes(), vertex_buffer.clone().into());

        let mut now = sync::now(dev.clone());
        now.cleanup_finished();
        let gpu_future = now.boxed();

        let _ = gpu_future
            .then_execute(graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None);

        vertex_buffer
    }

    fn create_index_buffer(
        dev: Arc<Device>,
        graphics_queue: Arc<Queue>,
        command_pool: &CommandPool,
        allocator: Arc<dyn MemoryAllocator>,
    ) -> Arc<Buffer> {
        let staging_buffer = Buffer::from_data(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                sharing: Sharing::Exclusive,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter {
                    required_flags: MemoryPropertyFlags::HOST_VISIBLE
                        | MemoryPropertyFlags::HOST_COHERENT,
                    ..Default::default()
                },
                ..Default::default()
            },
            INDICES,
        )
        .unwrap();

        let index_buffer = Buffer::new(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST | BufferUsage::INDEX_BUFFER,
                sharing: Sharing::Exclusive,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter {
                    required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
                    ..Default::default()
                },
                ..Default::default()
            },
            DeviceLayout::from_size_alignment(12, DeviceAlignment::MIN.into()).unwrap(),
        )
        .unwrap();

        let command_buffer =
            command_pool.record_copy_pass(staging_buffer.into_bytes(), index_buffer.clone().into());

        let mut now = sync::now(dev.clone());
        now.cleanup_finished();
        let gpu_future = now.boxed();

        let _ = gpu_future
            .then_execute(graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None);

        index_buffer
    }

    fn create_descriptor_set_layout(
        device: Arc<Device>,
    ) -> Result<Arc<DescriptorSetLayout>, Validated<vulkano::VulkanError>> {
        let ubo_layout_binding = DescriptorSetLayoutBinding {
            stages: ShaderStages::VERTEX,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
        };

        let layout_create_info = DescriptorSetLayoutCreateInfo {
            bindings: BTreeMap::from([(0, ubo_layout_binding)]),
            ..Default::default()
        };

        return DescriptorSetLayout::new(device, layout_create_info);
    }

    fn create_uniform_buffers(
        max_frames_in_flight: usize,
        allocator: Arc<dyn MemoryAllocator>,
    ) -> Result<Vec<Subbuffer<UniformBufferObject>>, Validated<VulkanError>> {
        let mut uniform_buffers: Vec<Subbuffer<UniformBufferObject>> =
            Vec::with_capacity(max_frames_in_flight);

        // TODO: convert to persistent mapping
        for _ in 0..max_frames_in_flight {
            let uniform_buffer = Buffer::new_sized::<UniformBufferObject>(
                allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    sharing: Sharing::Exclusive,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter {
                        required_flags: MemoryPropertyFlags::HOST_VISIBLE
                            | MemoryPropertyFlags::HOST_COHERENT,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();

            uniform_buffers.push(uniform_buffer.clone());
        }

        return Ok(uniform_buffers);
    }

    fn create_descriptor_sets(
        layout: Arc<DescriptorSetLayout>,
        uniform_buffers: &[Subbuffer<UniformBufferObject>], // Pass in the buffersr_sets(
        device: Arc<Device>,
    ) -> Vec<Arc<PersistentDescriptorSet>> {
        let descriptor_set_allocator =
            StandardDescriptorSetAllocator::new(device.clone(), Default::default());
        let num_sets = uniform_buffers.len();
        let mut descriptor_sets: Vec<Arc<PersistentDescriptorSet>> = Vec::with_capacity(num_sets);

        for i in 0..num_sets {
            let set = PersistentDescriptorSet::new(
                &descriptor_set_allocator,
                layout.clone(),
                [WriteDescriptorSet::buffer(0, uniform_buffers[i].clone())],
                [],
            )
            .unwrap(); // Handle potential errors
            descriptor_sets.push(set);
        }

        descriptor_sets
    }

    fn update_uniform_buffer(&mut self, image_index: usize, aspect_ratio: f32) {
        let elapsed_time = self.start_time.elapsed();

        let mut uniform_buffer = self.uniform_buffers[image_index].write().unwrap();

        uniform_buffer.model =
            Quaternion::new(Vector3::new(0.0, 0.0, 1.0), elapsed_time.as_secs_f32())
                .into_rotation_matrix();

        uniform_buffer.view = look_at(VECTOR3_ZERO, Vector3::new(0.0, elapsed_time.as_secs_f32().sin() * 5.0, 2.0));
        uniform_buffer.proj = perspective(60.0, aspect_ratio, 0.1, 100.0);
    }
}
