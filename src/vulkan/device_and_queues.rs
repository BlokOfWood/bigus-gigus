use std::{collections::HashSet, ffi::CStr, hash::RandomState, sync::Arc};

use ash::{
    khr::surface::Instance as SurfaceInstance,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, Format, FormatFeatureFlags, ImageTiling,
        MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceFeatures, Queue, QueueFlags, SurfaceKHR,
        TRUE,
    },
    Device, Entry, Instance,
};

use super::{swap_chain::SwapChainSupport, vulkan::REQUIRED_EXTENSIONS};

#[derive(Clone)]
pub struct BigusDevice {
    instance: Instance,
    pub phys_dev: PhysicalDevice,
    pub dev: Device,
    surface_instance: SurfaceInstance,
    queues: QueueFamilies,
}

impl BigusDevice {
    pub(super) fn new(entry: &Entry, instance: &Instance, surface: SurfaceKHR) -> Self {
        let surface_instance = SurfaceInstance::new(entry, instance);

        let physical_device = *unsafe {
            instance
                .enumerate_physical_devices()
                .unwrap()
                .iter()
                .find(|device| {
                    Self::is_device_suitable(
                        entry,
                        instance,
                        **device,
                        surface_instance.clone(),
                        surface,
                    )
                })
                .expect("No physical devices found")
        };

        let queue_family_indices = QueueFamilyIndices::find_queue_families(
            entry,
            instance,
            physical_device,
            surface.clone(),
        );

        println!(
            "Using device: {}",
            unsafe { instance.get_physical_device_properties(physical_device) }
                .device_name_as_c_str()
                .unwrap()
                .to_str()
                .unwrap()
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
            _presentation_queue: unsafe {
                device.get_device_queue(queue_family_indices.presentation_family.unwrap(), 0)
            },
        };

        Self {
            instance: instance.clone(),
            phys_dev: physical_device,
            dev: device,
            surface_instance,
            queues,
        }
    }

    fn is_device_suitable(
        entry: &Entry,
        instance: &Instance,
        device: PhysicalDevice,
        surface_instance: SurfaceInstance,
        surface: SurfaceKHR,
    ) -> bool {
        let queue_families =
            QueueFamilyIndices::find_queue_families(entry, instance, device, surface.clone());

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
        instance: &Instance,
        candidates: Vec<Format>,
        tiling: ImageTiling,
        features: FormatFeatureFlags,
    ) -> Format {
        for candidate in candidates {
            let format_props =
                unsafe { instance.get_physical_device_format_properties(self.phys_dev, candidate) };

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
}

#[derive(Clone)]
pub struct QueueFamilies {
    pub graphics_queue: Queue,
    pub _presentation_queue: Queue,
}
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub presentation_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn find_queue_families(
        entry: &Entry,
        inst: &Instance,
        device: PhysicalDevice,
        surface: SurfaceKHR,
    ) -> Self {
        let surf_instance = SurfaceInstance::new(entry, inst);

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
                    surf_instance
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
