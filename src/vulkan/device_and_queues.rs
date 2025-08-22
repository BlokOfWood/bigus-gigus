use std::{collections::HashSet, ffi::CStr, hash::RandomState};

use ash::{
    khr::surface::Instance as SurfaceInstance,
    vk::{
        api_version_major, api_version_minor, DeviceCreateInfo, DeviceQueueCreateInfo, Fence,
        FenceCreateFlags, FenceCreateInfo, Format, FormatFeatureFlags, ImageTiling,
        MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceProperties,
        Queue, QueueFlags, Semaphore, SemaphoreCreateInfo, SurfaceKHR, TRUE,
    },
    Device, Instance,
};

use crate::vulkan::vulkan::MAX_FRAMES_IN_FLIGHT;

use super::{swap_chain::SwapChainSupport, vulkan::REQUIRED_EXTENSIONS};

#[derive(Clone)]
pub struct BigusDevice {
    pub instance: Instance,
    pub phys_dev: PhysicalDevice,
    pub phys_dev_capabilities: PhysicalDeviceProperties,
    pub dev: Device,
    pub swapchain_dev: ash::khr::swapchain::Device,
    pub queues: QueueFamilies,
    pub queue_family_indices: QueueFamilyIndices,
}

impl BigusDevice {
    pub(super) fn new(instance: &Instance, surface: SurfaceKHR, surface_instance: SurfaceInstance) -> Self {
        let physical_device = *unsafe {
            instance
                .enumerate_physical_devices()
                .unwrap()
                .iter()
                .find(|device| {
                    Self::is_device_suitable(
                        instance,
                        **device,
                        &surface_instance,
                        surface,
                    )
                })
                .expect("No physical devices found")
        };

        let physical_device_capabilities =
            unsafe { instance.get_physical_device_properties(physical_device) };

        let queue_family_indices = QueueFamilyIndices::find_queue_families(
            instance,
            physical_device,
            surface.clone(),
            &surface_instance
        );

        let device_capabilities =
            unsafe { instance.get_physical_device_properties(physical_device) };

        let device_name = device_capabilities
            .device_name_as_c_str()
            .unwrap()
            .to_str()
            .unwrap();

        let device_version = device_capabilities.api_version;

        println!(
            "Using device: {}, Version: {}.{}",
            device_name,
            api_version_major(device_version),
            api_version_minor(device_version)
        );

        let unique_queue_families: HashSet<u32, RandomState> = HashSet::from_iter([
            queue_family_indices.graphics_family.unwrap(),
            queue_family_indices.presentation_family.unwrap(),
        ]);

        let queue_create_infos: Vec<DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .map(|queue_family_index| DeviceQueueCreateInfo {
                queue_family_index: queue_family_index.clone(),
                queue_count: 1,
                p_queue_priorities: &1.0,
                ..Default::default()
            })
            .collect();

        let device_features = PhysicalDeviceFeatures {
            sampler_anisotropy: TRUE,
            ..Default::default()
        };

        let required_extensions =
            REQUIRED_EXTENSIONS.map(|extension| extension.as_ptr() as *const i8);

        let create_info = DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &device_features,
            enabled_extension_count: REQUIRED_EXTENSIONS.len() as u32,
            pp_enabled_extension_names: required_extensions.as_ptr(),
            ..Default::default()
        };

        let device = unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .unwrap()
        };

        let queues = QueueFamilies {
            graphics_queue: unsafe {
                device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0)
            },
            presentation_queue: unsafe {
                device.get_device_queue(queue_family_indices.presentation_family.unwrap(), 0)
            },
        };

        let swapchain_dev = ash::khr::swapchain::Device::new(instance, &device);

        Self {
            instance: instance.clone(),
            phys_dev: physical_device,
            phys_dev_capabilities: physical_device_capabilities,
            swapchain_dev,
            dev: device,
            queues,
            queue_family_indices
        }
    }

    fn is_device_suitable(
        instance: &Instance,
        device: PhysicalDevice,
        surface_instance: &SurfaceInstance,
        surface: SurfaceKHR,
    ) -> bool {
        let queue_families =
            QueueFamilyIndices::find_queue_families(instance, device, surface.clone(), &surface_instance);

        let swap_chain_support =
            SwapChainSupport::new(device.clone(), surface_instance, surface.clone());

        return queue_families.has_required_families()
            && Self::check_device_extension_support(instance, device)
            && !swap_chain_support.formats.is_empty()
            && !swap_chain_support.present_modes.is_empty();
    }

    fn check_device_extension_support(instance: &Instance, device: PhysicalDevice) -> bool {
        let device_extension_properties =
            unsafe { instance.enumerate_device_extension_properties(device) }.unwrap();
        let extension_names: Vec<&CStr> = device_extension_properties
            .iter()
            .map(|extension| extension.extension_name_as_c_str().unwrap())
            .collect();

        return REQUIRED_EXTENSIONS
            .iter()
            .all(|extension| extension_names.contains(extension));
    }

    pub fn find_supported_format(
        &self,
        candidates: Vec<Format>,
        tiling: ImageTiling,
        features: FormatFeatureFlags,
    ) -> Format {
        for candidate in candidates {
            let format_props =
                unsafe { self.instance.get_physical_device_format_properties(self.phys_dev, candidate) };

            match tiling {
                ImageTiling::LINEAR if format_props.linear_tiling_features.contains(features) => {
                    return candidate
                }
                ImageTiling::OPTIMAL if format_props.optimal_tiling_features.contains(features) => {
                    return candidate
                }
                _ => (),
            };
        }

        panic!("Couldn't find format");
    }

    pub fn find_memory_type(
        &self,
        instance: &Instance,
        type_filter: u32,
        properties: MemoryPropertyFlags,
    ) -> u32 {
        let phys_dev_mem_props =
            unsafe { instance.get_physical_device_memory_properties(self.phys_dev) };

        for (i, mem_type) in phys_dev_mem_props.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) != 0 && mem_type.property_flags.contains(properties) {
                return i as u32;
            }
        }

        panic!("Failed to find suitable memory type!");
    }

    pub fn create_sync_objects(&self) -> (Vec<Semaphore>, Vec<Semaphore>, Vec<Fence>) {
        let mut image_available_semaphores: Vec<Semaphore> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores: Vec<Semaphore> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences: Vec<Fence> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_info = SemaphoreCreateInfo {
            ..Default::default()
        };

        let fence_info = FenceCreateInfo {
            flags: FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_semaphores.push(self.dev.create_semaphore(&semaphore_info, None).unwrap());
                render_finished_semaphores.push(self.dev.create_semaphore(&semaphore_info, None).unwrap());
                in_flight_fences.push(self.dev.create_fence(&fence_info, None).unwrap()); 
            }
        }

        (image_available_semaphores, render_finished_semaphores, in_flight_fences)
    }
}

#[derive(Clone)]
pub struct QueueFamilies {
    pub graphics_queue: Queue,
    pub presentation_queue: Queue,
}
#[derive(Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub presentation_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn find_queue_families(
        inst: &Instance,
        device: PhysicalDevice,
        surface: SurfaceKHR,
        surface_instance: &SurfaceInstance,
    ) -> Self {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            presentation_family: None,
        };

        let device_queue_family_properties =
            unsafe { inst.get_physical_device_queue_family_properties(device) };

        device_queue_family_properties
            .iter()
            .enumerate()
            .for_each(|(i, queue_family)| {
                if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                    indices.graphics_family = Some(i as u32);
                }
                if unsafe {
                    surface_instance
                        .get_physical_device_surface_support(device, i as u32, surface)
                        .unwrap()
                } {
                    indices.presentation_family = Some(i as u32);
                }
            });

        indices
    }

    pub fn has_required_families(&self) -> bool {
        self.graphics_family.is_some() && self.presentation_family.is_some()
    }
}
