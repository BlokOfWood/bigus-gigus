use std::sync::Arc;

use vulkano::{
    device::{physical::PhysicalDevice, Queue, QueueFlags},
    swapchain::Surface,
};

pub struct QueueFamilies {
    pub graphics_queue: Arc<Queue>,
    pub presentation_queue: Arc<Queue>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub presentation_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn find_queue_families(device: Arc<PhysicalDevice>, surface: Arc<Surface>) -> Self {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            presentation_family: None,
        };

        device
            .queue_family_properties()
            .iter()
            .enumerate()
            .for_each(|(i, queue_family)| {
                if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                    indices.graphics_family = Some(i as u32);
                }
                if device.surface_support(i as u32, &surface).unwrap() {
                    indices.presentation_family = Some(i as u32);
                }
            });

        indices
    }

    pub fn has_required_families(&self) -> bool {
        self.graphics_family.is_some() && self.presentation_family.is_some()
    }
}
