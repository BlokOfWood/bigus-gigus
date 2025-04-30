use std::sync::Arc;

use vulkano::{
    device::{physical::PhysicalDevice, Device}, format::Format, image::{view::ImageView, Image, ImageUsage}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, swapchain::{
        ColorSpace, CompositeAlpha, PresentMode, Surface, SurfaceCapabilities, SurfaceInfo,
        Swapchain, SwapchainCreateInfo,
    }, sync::Sharing
};
use winit::window::Window;

use super::{
    device_and_queues::QueueFamilyIndices, texture::create_image_views, vulkan::VulkanRenderer,
};

pub struct SwapChainSupport {
    pub capabilities: SurfaceCapabilities,
    pub formats: Vec<(Format, ColorSpace)>,
    pub present_modes: Vec<PresentMode>,
}

impl SwapChainSupport {
    pub fn new(phys_dev: Arc<PhysicalDevice>, surface: Arc<Surface>) -> Self {
        let capabilities = phys_dev
            .surface_capabilities(
                Arc::as_ref(&surface),
                vulkano::swapchain::SurfaceInfo::default(),
            )
            .unwrap();

        let formats = phys_dev
            .surface_formats(
                Arc::as_ref(&surface),
                vulkano::swapchain::SurfaceInfo::default(),
            )
            .unwrap();

        let present_modes = phys_dev
            .surface_present_modes(
                Arc::as_ref(&surface),
                vulkano::swapchain::SurfaceInfo::default(),
            )
            .unwrap();

        SwapChainSupport {
            capabilities,
            formats,
            present_modes: present_modes.collect(),
        }
    }

    pub fn choose_surface_format(&self) -> (Format, ColorSpace) {
        let optimal_format_result = self.formats.iter().find(|format| {
            format.0 == Format::B8G8R8A8_SRGB && format.1 == ColorSpace::SrgbNonLinear
        });

        if let Some(optimal_format) = optimal_format_result {
            optimal_format.clone()
        } else {
            self.formats.first().unwrap().clone()
        }
    }

    pub fn choose_present_mode(&self) -> PresentMode {
        let optimal_present_result = self
            .present_modes
            .iter()
            .find(|format| **format == PresentMode::Mailbox);

        if let Some(optimal_present) = optimal_present_result {
            optimal_present.clone()
        } else {
            PresentMode::Fifo
        }
    }
}

pub fn create_swap_chain(
    dev: Arc<Device>,
    surface: Arc<Surface>,
    capabilities: SurfaceCapabilities,
    window: Arc<Window>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>, Format, [u32; 2]) {
    let phys_dev = dev.clone().physical_device().clone();

    let swap_chain_support = SwapChainSupport::new(phys_dev.clone(), surface.clone());

    let surface_format = swap_chain_support.choose_surface_format();
    let present_mode = swap_chain_support.choose_present_mode();
    let swap_extent = choose_swap_extent(capabilities, window.clone());

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

pub(super) fn create_frame_buffers(
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

impl VulkanRenderer {
    pub fn recreate_swap_chain(&mut self, window: Arc<Window>) {
        let surface = Surface::from_window(self.inst.clone(), window.clone())
            .expect("Failed to create surface from window.");

        self.surface = surface.clone();

        let swap_extent = choose_swap_extent(
            self.device
                .physical_device()
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
        );
    }
}
