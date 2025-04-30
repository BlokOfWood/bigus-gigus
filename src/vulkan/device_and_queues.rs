use std::sync::Arc;

use vulkano::{
    device::{physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, instance::Instance, swapchain::Surface
};

use super::swap_chain::SwapChainSupport;

pub(super) fn create_vulkan_physical_device(
    vk_instance: Arc<Instance>,
    vk_surface: Arc<Surface>,
) -> Arc<PhysicalDevice> {
    let vk_phys_dev = vk_instance
        .enumerate_physical_devices()
        .unwrap()
        .find(|device| is_device_suitable(device, vk_surface.clone()))
        .expect("No physical devices found");

    println!("Using device: {}", vk_phys_dev.properties().device_name);

    vk_phys_dev
}

pub(super) fn create_device_and_queues(
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

    let enabled_features = Features {
        sampler_anisotropy: true,
        ..Default::default()
    };

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
        .find(|queue| queue.queue_family_index() == queue_family_indicies.graphics_family.unwrap())
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

fn is_device_suitable(device: &Arc<PhysicalDevice>, surface: Arc<Surface>) -> bool {
    let queue_families = QueueFamilyIndices::find_queue_families(device.clone(), surface.clone());
    let swap_chain_support = SwapChainSupport::new(device.clone(), surface.clone());

    return queue_families.has_required_families()
        && device.supported_extensions().khr_swapchain
        && !swap_chain_support.formats.is_empty()
        && !swap_chain_support.present_modes.is_empty();
}

pub struct QueueFamilies {
    pub graphics_queue: Arc<Queue>,
    pub _presentation_queue: Arc<Queue>,
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
