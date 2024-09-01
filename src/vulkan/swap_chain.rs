use std::sync::Arc;

use vulkano::{
    device::physical::PhysicalDevice, format::Format, swapchain::{ColorSpace, PresentMode, Surface, SurfaceCapabilities}
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
