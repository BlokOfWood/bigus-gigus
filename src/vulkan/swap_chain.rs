use ash::khr::surface::Instance as SurfaceInstance;
use ash::vk::{
    self, ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, Framebuffer,
    FramebufferCreateInfo, Image, ImageUsageFlags, ImageView, PhysicalDevice, PresentModeKHR,
    RenderPass, SharingMode, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR,
    SwapchainCreateInfoKHR, SwapchainKHR, TRUE,
};
use ash::Instance;

use super::device_and_queues::{BigusDevice, QueueFamilyIndices};

pub struct SwapChainSupport {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapChainSupport {
    pub fn new(
        physical_device: PhysicalDevice,
        surface_instance: SurfaceInstance,
        surface: SurfaceKHR,
    ) -> Self {
        let capabilities = unsafe {
            surface_instance.get_physical_device_surface_capabilities(physical_device, surface)
        }
        .unwrap();
        let formats = unsafe {
            surface_instance.get_physical_device_surface_formats(physical_device, surface)
        }
        .unwrap();

        let present_modes = unsafe {
            surface_instance.get_physical_device_surface_present_modes(physical_device, surface)
        }
        .unwrap();

        SwapChainSupport {
            capabilities,
            formats,
            present_modes,
        }
    }

    pub fn choose_surface_format(&self) -> SurfaceFormatKHR {
        let optimal_format_result = self.formats.iter().find(|format| {
            format.format == Format::B8G8R8A8_SRGB
                && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
        });

        if let Some(optimal_format) = optimal_format_result {
            optimal_format.clone()
        } else {
            self.formats.first().unwrap().clone()
        }
    }

    pub fn choose_present_mode(&self) -> PresentModeKHR {
        let optimal_present_result = self
            .present_modes
            .iter()
            .find(|format| **format == PresentModeKHR::MAILBOX);

        if let Some(optimal_present) = optimal_present_result {
            optimal_present.clone()
        } else {
            PresentModeKHR::FIFO
        }
    }

    pub fn choose_swap_extent(&self, window_extent: [u32; 2]) -> Extent2D {
        let capabilities = self.capabilities;

        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            Extent2D {
                width: window_extent[0].clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: window_extent[1].clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}

pub fn create_swap_chain(
    device: ash::khr::swapchain::Device,
    physical_device: PhysicalDevice,
    surface_instance: SurfaceInstance,
    surface: SurfaceKHR,
    window_extent: [u32; 2],
    queue_family_indices: &QueueFamilyIndices,
) -> (SwapchainKHR, Vec<Image>, SurfaceFormatKHR, Extent2D) {
    let swap_chain_support = SwapChainSupport::new(physical_device, surface_instance, surface);

    let surface_format = swap_chain_support.choose_surface_format();
    let present_mode = swap_chain_support.choose_present_mode();
    let swap_extent = swap_chain_support.choose_swap_extent(window_extent);

    let min_image_count = swap_chain_support.capabilities.min_image_count;
    let max_image_count = swap_chain_support.capabilities.max_image_count;

    let image_count = if max_image_count > 0 && min_image_count + 1 > max_image_count {
        max_image_count
    } else {
        min_image_count + 1
    };

    let mut create_info = SwapchainCreateInfoKHR {
        surface,
        min_image_count: image_count,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: swap_extent,
        image_array_layers: 1,
        image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
        pre_transform: swap_chain_support.capabilities.current_transform,
        composite_alpha: CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: TRUE,
        ..Default::default()
    };

    if queue_family_indices.graphics_family != queue_family_indices.presentation_family {
        create_info.image_sharing_mode = SharingMode::CONCURRENT;
        create_info.queue_family_index_count = 2;
        create_info.p_queue_family_indices = [
            queue_family_indices.graphics_family.unwrap(),
            queue_family_indices.presentation_family.unwrap(),
        ]
        .as_ptr();
    } else {
        create_info.image_sharing_mode = SharingMode::EXCLUSIVE;
    }

    let swap_chain = unsafe { device.create_swapchain(&create_info, None).unwrap() };
    let swap_chain_images = unsafe { device.get_swapchain_images(swap_chain).unwrap() };

    (swap_chain, swap_chain_images, surface_format, swap_extent)
}

pub(super) fn create_frame_buffers(
    dev: &BigusDevice,
    render_pass: RenderPass,
    swap_chain_image_views: Vec<ImageView>,
    depth_image_view: ImageView,
    swap_chain_extent: Extent2D,
) -> Vec<Framebuffer> {
    let mut frame_buffers = Vec::new();

    for i in 0..swap_chain_image_views.len() {
        let attachments = [swap_chain_image_views[i], depth_image_view];

        let frame_buffer = unsafe {
            dev.dev.create_framebuffer(
                &FramebufferCreateInfo {
                    render_pass,
                    attachment_count: attachments.len() as u32,
                    p_attachments: attachments.as_ptr(),
                    width: swap_chain_extent.width,
                    height: swap_chain_extent.height,
                    layers: 1,
                    ..Default::default()
                },
                None,
            )
        };

        frame_buffers.push(frame_buffer.unwrap());
    }

    frame_buffers
}

/*
                impl VulkanRenderer {
                    pub fn recreate_swap_chain(&mut self, window: Arc<Window>) {
                        let surface = Surface::from_window(self.inst.clone(), window.clone())
                        .expect("Failed to create surface from window.");

                    self.surface = surface.clone();

                    let swap_extent = choose_swap_extent(
                        self.device
                .phys_device()
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

        self.image_views = create_image_views(&self.images, self.image_format);

        self.frame_buffers = create_frame_buffers(
            self.render_pass.clone(),
            &self.image_views,
            self.image_extent,
            self.depth_image_view.clone()
        );
    }
}
*/
