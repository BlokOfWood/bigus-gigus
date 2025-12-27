use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use ash::{
    vk::{
        self, ApplicationInfo, Buffer, ClearColorValue, ClearDepthStencilValue, ClearValue,
        CommandBuffer, CommandBufferBeginInfo, CommandBufferResetFlags, DescriptorSet,
        DeviceMemory, Extent2D, Fence, Framebuffer, Image, ImageView, IndexType, Offset2D,
        Pipeline, PipelineBindPoint, PipelineLayout, PipelineStageFlags, PresentInfoKHR, Rect2D,
        RenderPass, RenderPassBeginInfo, Result, Semaphore, SubmitInfo, SubpassContents,
        SurfaceKHR, SwapchainKHR, Viewport, KHR_SWAPCHAIN_NAME,
    },
    Entry, Instance,
};
use winit::{
    event::{ElementState, KeyEvent},
    event_loop::ActiveEventLoop,
    keyboard::Key,
    raw_window_handle::HasDisplayHandle,
    window::Window,
};

use crate::{
    ecs::{builtins::mesh::Mesh, ecs_runner::EcsRunner},
    resource_handling::{resource_handler::ResourceHandler, resources::mesh::Model},
    vulkan::{
        command_pool::create_render_pass,
        device_and_queues::QueueFamilyIndices,
        pipeline::{create_descriptor_set_layout, create_graphics_pipeline},
        window::{create_surface, enumerate_required_extensions},
    },
};

use ash::khr::surface::Instance as SurfaceInstance;

use crate::vulkan::device_and_queues::BigusDevice;

const ENGINE_NAME: &str = "Very cool engine";
const APPLICATION_NAME: &str = "Very cool application";
pub(super) const REQUIRED_EXTENSIONS: [&CStr; 1] = [KHR_SWAPCHAIN_NAME];

pub(super) const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanRenderer {
    pub(super) entry: Entry,
    pub(super) instance: Instance,
    pub(super) device: BigusDevice,
    pub(super) swapchain: SwapchainKHR,
    pub(super) swapchain_extent: Extent2D,

    pub(super) graphics_pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
    descriptor_sets: Vec<DescriptorSet>,

    pub(super) render_pass: RenderPass,
    pub(super) window: Arc<Window>,
    pub(super) surface_instance: SurfaceInstance,
    pub(super) surface: SurfaceKHR,

    //pub mip_levels: u32,
    pub(super) color_image: Image,
    pub(super) color_image_view: ImageView,
    pub(super) color_image_memory: DeviceMemory,

    pub(super) depth_image: Image,
    pub(super) depth_image_view: ImageView,
    pub(super) depth_image_memory: DeviceMemory,

    pub(super) framebuffers: Vec<Framebuffer>,
    pub(super) swapchain_image_views: Vec<ImageView>,
    pub(super) _uniform_buffers: Vec<Buffer>,
    pub(super) uniform_buffers_mapped: Vec<*mut c_void>,
    current_frame: u32,
    in_flight_fences: Vec<Fence>,
    image_available_semaphores: Vec<Semaphore>,
    render_finished_semaphores: Vec<Semaphore>,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,

    //pub(super) start_time: Instant,
    pub(super) command_buffers: Vec<CommandBuffer>,

    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl VulkanRenderer {
    pub fn new(
        window: Arc<Window>,
        event_loop: &ActiveEventLoop,
        resource_handler: &mut ResourceHandler,
        ecs: &mut EcsRunner,
    ) -> Self {
        let room_model_handle = resource_handler
            .load_resource("assets/models/viking_room.obj")
            .unwrap();
        let model: &Model = resource_handler
            .retrieve_resource(&room_model_handle)
            .unwrap();

        let entity = ecs.create_entity();
        entity.add_component(
            ecs,
            Mesh {
                indices: model.indices.clone(),
                vertices: model.vertices.clone(),
            },
        );
        let meshes = ecs.query::<Mesh>();

        for mesh in meshes {
            println!(
                "Mesh - Vertices: {}, Indices: {}",
                mesh.vertices.len(),
                mesh.indices.len()
            );
        }

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
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };

        let window_required_extensions =
            enumerate_required_extensions(event_loop.display_handle().unwrap().into())
                .expect("Failed to enumerate required extensions.");

        let mut enabled_extensions = Vec::new();
        window_required_extensions
            .iter()
            .for_each(|extension| enabled_extensions.push(*extension));

        let mut enabled_layers_cnames = Vec::new();

        if cfg!(debug_assertions) {
            println!("Enabling validation layers");
            let cstr = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
            enabled_layers_cnames.push(cstr); // Store the CString so it lives long enough
        }

        let enabled_layers: Vec<*const i8> = enabled_layers_cnames
            .iter()
            .map(|layer| layer.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo {
            enabled_layer_count: enabled_layers.len() as u32,
            pp_enabled_layer_names: enabled_layers.as_ptr(),
            enabled_extension_count: enabled_extensions.len() as u32,
            pp_enabled_extension_names: enabled_extensions.as_ptr(),
            p_application_info: &application_info,
            ..Default::default()
        };

        let instance = unsafe {
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
            create_surface(&entry, &instance, window.clone(), None)
                .expect("Failed to create surface from window.")
        };

        let surface_instance = SurfaceInstance::new(&entry, &instance);

        let bigus_device = BigusDevice::new(&instance, surface, surface_instance.clone());

        let queue_family_indices = QueueFamilyIndices::find_queue_families(
            &instance,
            bigus_device.phys_dev,
            surface,
            &surface_instance,
        );

        let (swapchain, images, image_format, image_extent) = bigus_device.create_swap_chain(
            &surface_instance,
            surface,
            [window.inner_size().width, window.inner_size().height],
            &queue_family_indices,
        );

        let descriptor_set_layout = create_descriptor_set_layout(&bigus_device);

        let command_pool = bigus_device.create_command_pool(&queue_family_indices);

        let (image, mip_levels) =
            bigus_device.create_texture_image("assets/textures/viking_room.png", &command_pool);

        let image_views = bigus_device.create_image_views(images, image_format.format, 1);

        let (color_image, color_image_view, color_image_memory) = bigus_device
            .create_color_resources(
                [image_extent.width, image_extent.height],
                image_format.format,
            );

        let (depth_image, depth_image_view, depth_image_memory, depth_image_format) =
            bigus_device.create_depth_resources([image_extent.width, image_extent.height]);

        let render_pass =
            create_render_pass(&bigus_device, image_format.format, depth_image_format);

        let (graphics_pipeline, pipeline_layout) =
            create_graphics_pipeline(&bigus_device, render_pass, vec![descriptor_set_layout]);

        let framebuffers = bigus_device.create_frame_buffers(
            render_pass.clone(),
            color_image_view,
            &image_views,
            depth_image_view.clone(),
            image_extent,
        );

        let image_view = bigus_device.create_texture_image_view(image, mip_levels);

        let image_sampler = bigus_device.create_texture_sampler();

        let (vertex_buffer, _vertex_buffer_memory) =
            bigus_device.create_vertex_buffer(&model.vertices, &command_pool);

        let (index_buffer, _index_buffer_memory) =
            bigus_device.create_index_buffer(&model.indices, &command_pool);

        let (uniform_buffers, _uniform_buffer_memories, uniform_buffers_mapped) =
            bigus_device.create_uniform_buffers();

        let descriptor_pool = bigus_device.create_descriptor_pool();

        let descriptor_sets = bigus_device.create_descriptor_sets(
            descriptor_set_layout,
            descriptor_pool,
            &uniform_buffers,
            image_view,
            image_sampler,
        );

        let command_buffers = bigus_device.create_command_buffers(command_pool);

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            bigus_device.create_sync_objects();

        VulkanRenderer {
            entry,
            instance,
            device: bigus_device,
            surface,
            surface_instance,
            window,
            graphics_pipeline,
            pipeline_layout,
            descriptor_sets,
            render_pass,
            swapchain,
            vertex_buffer,
            index_buffer,
            index_count: model.indices.len() as u32,
            //mip_levels,
            color_image,
            color_image_memory,
            color_image_view,

            depth_image,
            depth_image_memory,
            depth_image_view,

            framebuffers,
            swapchain_image_views: image_views,
            swapchain_extent: image_extent,
            current_frame: 0,
            in_flight_fences,
            image_available_semaphores,
            render_finished_semaphores,
            _uniform_buffers: uniform_buffers,
            uniform_buffers_mapped,
            //start_time: Instant::now(),
            command_buffers,
            x: -2.0,
            y: -2.0,
            z: 3.0,
        }
    }

    pub(super) fn record_command_buffer(
        &self,
        command_buffer: &CommandBuffer,
        framebuffer: &Framebuffer,
    ) {
        unsafe {
            self.device
                .dev
                .begin_command_buffer(
                    *command_buffer,
                    &CommandBufferBeginInfo {
                        ..Default::default()
                    },
                )
                .unwrap();
        };

        let clear_values = [
            ClearValue {
                color: ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            ClearValue {
                depth_stencil: ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        unsafe {
            self.device.dev.cmd_begin_render_pass(
                *command_buffer,
                &RenderPassBeginInfo {
                    render_pass: self.render_pass,
                    framebuffer: *framebuffer,
                    render_area: Rect2D {
                        offset: Offset2D { x: 0, y: 0 },
                        extent: self.swapchain_extent,
                    },
                    clear_value_count: clear_values.len() as u32,
                    p_clear_values: clear_values.as_ptr(),
                    ..Default::default()
                },
                SubpassContents::INLINE,
            );

            self.device.dev.cmd_bind_pipeline(
                *command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );

            self.device.dev.cmd_set_viewport(
                *command_buffer,
                0,
                &[Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: self.swapchain_extent.width as f32,
                    height: self.swapchain_extent.height as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );

            self.device.dev.cmd_set_scissor(
                *command_buffer,
                0,
                &[Rect2D {
                    offset: Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_extent,
                }],
            );

            self.device.dev.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                &[self.vertex_buffer],
                &[0],
            );

            self.device.dev.cmd_bind_index_buffer(
                *command_buffer,
                self.index_buffer,
                0,
                IndexType::UINT32,
            );

            self.device.dev.cmd_bind_descriptor_sets(
                *command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[self.current_frame as usize]],
                &[],
            );

            self.device
                .dev
                .cmd_draw_indexed(*command_buffer, self.index_count, 1, 0, 0, 0);

            self.device.dev.cmd_end_render_pass(*command_buffer);

            self.device.dev.end_command_buffer(*command_buffer).unwrap();
        };
    }

    pub fn process_key_event(&mut self, event: KeyEvent) {
        if event.state == ElementState::Released {
            return;
        }

        let key = match event.logical_key {
            Key::Character(key) => key,
            _ => return,
        };

        match key.as_str() {
            "w" => self.y += 0.1,
            "s" => self.y -= 0.1,
            "a" => self.x -= 0.1,
            "d" => self.x += 0.1,
            "q" => self.z -= 0.1,
            "e" => self.z += 0.1,
            _ => return,
        }

        println!(
            "{} Position - x: {}, y: {}, z: {}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self.x,
            self.y,
            self.z
        );
    }

    pub fn draw_frame(&mut self) {
        // Acquires an image from the swap chain to draw unto.
        // The swap chain is basically a buffer of images, where one of them is being displayed while we draw unto the other one.
        // Basically decouples presenting an image from drawing the image, so that we can sync with the monitor's refresh rate.

        unsafe {
            self.device
                .dev
                .wait_for_fences(
                    &[self.in_flight_fences[self.current_frame as usize]],
                    true,
                    1_000_000_000,
                )
                .unwrap()
        };

        let result = unsafe {
            self.device.swapchain_dev.acquire_next_image(
                self.swapchain,
                1 * 1_000_000_000,
                self.image_available_semaphores[self.current_frame as usize],
                Fence::null(),
            )
        };

        let (image_index, _is_suboptimal) = match result {
            Ok((image_idx, is_suboptimal)) => (image_idx, is_suboptimal),
            Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swap_chain();
                return;
            }
            Err(err) => panic!("Failed to acquire swap chain image! Error: {}", err),
        };

        let aspect_ratio =
            self.window.inner_size().width as f32 / self.window.inner_size().height as f32;
        self.update_uniform_buffer(self.current_frame as usize, aspect_ratio);

        unsafe {
            self.device
                .dev
                .reset_fences(&[self.in_flight_fences[self.current_frame as usize]])
                .unwrap();
        }

        unsafe {
            self.device
                .dev
                .reset_command_buffer(
                    self.command_buffers[self.current_frame as usize],
                    CommandBufferResetFlags::empty(),
                )
                .unwrap();
            self.record_command_buffer(
                &self.command_buffers[self.current_frame as usize],
                &self.framebuffers[self.current_frame as usize],
            );

            self.device
                .dev
                .queue_submit(
                    self.device.queues.graphics_queue,
                    &[SubmitInfo {
                        wait_semaphore_count: 1,
                        p_wait_semaphores: &self.image_available_semaphores
                            [self.current_frame as usize],
                        p_wait_dst_stage_mask: &PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,

                        command_buffer_count: 1,
                        p_command_buffers: &self.command_buffers[self.current_frame as usize],

                        signal_semaphore_count: 1,
                        p_signal_semaphores: &self.render_finished_semaphores
                            [self.current_frame as usize],
                        ..Default::default()
                    }],
                    self.in_flight_fences[self.current_frame as usize],
                )
                .unwrap();

            match self.device.swapchain_dev.queue_present(
                self.device.queues.presentation_queue,
                &PresentInfoKHR {
                    wait_semaphore_count: 1,
                    p_wait_semaphores: &self.render_finished_semaphores
                        [self.current_frame as usize],

                    swapchain_count: 1,
                    p_swapchains: &self.swapchain,

                    p_image_indices: &image_index,

                    ..Default::default()
                },
            ) {
                Ok(_) => {}
                Err(Result::ERROR_OUT_OF_DATE_KHR) | Err(Result::SUBOPTIMAL_KHR) => {
                    self.recreate_swap_chain();
                    println!("recreate");
                }
                Err(e) => panic!("Failed to present swap chain image. Error: {}", e),
            }

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT as u32;
        }
    }
}
