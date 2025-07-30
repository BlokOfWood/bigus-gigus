use std::ffi::c_void;

use ash::{
    vk::{
        AccessFlags, BorderColor, BufferUsageFlags, CommandPool, CompareOp, DependencyFlags, DeviceMemory, Extent3D, Filter, Format, FormatFeatureFlags, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, PipelineStageFlags, SampleCountFlags, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode, SharingMode, FALSE, QUEUE_FAMILY_IGNORED, TRUE
    },
    Instance,
};

use super::device_and_queues::BigusDevice;

pub fn create_texture_image(device: &BigusDevice, command_pool: &CommandPool) -> Image {
    let open_image = image::open("assets/textures/statue.jpg").unwrap();

    let image_extent = [open_image.width(), open_image.height()];

    let image_data = open_image.to_rgba8();

    let (staging_buffer, staging_buffer_memory) = device.create_buffer(
        image_data.len() as u64,
        BufferUsageFlags::TRANSFER_SRC,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let data = device
            .dev
            .map_memory(
                staging_buffer_memory,
                0,
                image_data.len() as u64,
                MemoryMapFlags::empty(),
            )
            .unwrap();
        data.copy_from(image_data.as_ptr() as *mut c_void, image_data.len());

        device.dev.unmap_memory(staging_buffer_memory);
    };

    drop(open_image);

    let (texture_image, texture_image_memory) = device.create_image(
        image_extent,
        Format::R8G8B8A8_SRGB,
        ImageTiling::OPTIMAL,
        ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED,
        MemoryPropertyFlags::DEVICE_LOCAL,
    );

    transition_image_layout(
        device,
        command_pool,
        ImageLayout::UNDEFINED,
        ImageLayout::TRANSFER_DST_OPTIMAL,
        texture_image,
    );

    device.copy_buffer_to_image(
        staging_buffer,
        texture_image,
        command_pool,
        image_extent[0],
        image_extent[1],
    );

    transition_image_layout(
        device,
        command_pool,
        ImageLayout::TRANSFER_DST_OPTIMAL,
        ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        texture_image,
    );

    unsafe {
        device.dev.destroy_buffer(staging_buffer, None);
        device.dev.free_memory(staging_buffer_memory, None);
    };

    texture_image
}

fn transition_image_layout(
    device: &BigusDevice,
    command_pool: &CommandPool,
    old_layout: ImageLayout,
    new_layout: ImageLayout,
    image: Image,
) {
    let command_buffer = device.begin_single_time_commands(command_pool);

    let barrier = ImageMemoryBarrier {
        old_layout,
        new_layout,
        src_queue_family_index: QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    if old_layout == ImageLayout::UNDEFINED && new_layout == ImageLayout::TRANSFER_DST_OPTIMAL {
        unsafe {
            device.dev.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::TOP_OF_PIPE,
                PipelineStageFlags::TRANSFER,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier
                    .src_access_mask(AccessFlags::empty())
                    .dst_access_mask(AccessFlags::TRANSFER_WRITE)],
            )
        };
    } else {
        unsafe {
            device.dev.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::FRAGMENT_SHADER,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier
                    .src_access_mask(AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(AccessFlags::SHADER_READ)],
            )
        }
    }

    device.end_single_time_commands(command_pool, command_buffer);
}
pub fn create_texture_image_view(device: &BigusDevice, image: Image) -> ImageView {
    device.create_image_view(image, Format::R8G8B8A8_SRGB, ImageAspectFlags::COLOR)
}

pub fn create_texture_sampler(device: &BigusDevice) -> Sampler {
    unsafe {
        device
            .dev
            .create_sampler(
                &SamplerCreateInfo {
                    mag_filter: Filter::LINEAR,
                    min_filter: Filter::LINEAR,
                    address_mode_u: SamplerAddressMode::REPEAT,
                    address_mode_v: SamplerAddressMode::REPEAT,
                    address_mode_w: SamplerAddressMode::REPEAT,
                    anisotropy_enable: TRUE,
                    max_anisotropy: device.phys_dev_capabilities.limits.max_sampler_anisotropy,
                    border_color: BorderColor::INT_OPAQUE_BLACK,
                    unnormalized_coordinates: FALSE,
                    compare_enable: FALSE,
                    compare_op: CompareOp::ALWAYS,
                    mipmap_mode: SamplerMipmapMode::LINEAR,
                    ..Default::default()
                },
                None,
            )
            .unwrap()
    }
}

impl BigusDevice {
    pub fn create_image(
        &self,
        extent: [u32; 2],
        format: Format,
        tiling: ImageTiling,
        usage: ImageUsageFlags,
        properties: MemoryPropertyFlags,
    ) -> (Image, DeviceMemory) {
        let image_info = ImageCreateInfo {
            image_type: ImageType::TYPE_2D,
            extent: Extent3D {
                width: extent[0],
                height: extent[1],
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format,
            tiling,
            initial_layout: ImageLayout::UNDEFINED,
            usage,
            samples: SampleCountFlags::TYPE_1,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let image = unsafe { self.dev.create_image(&image_info, None).unwrap() };

        let mem_requirements = unsafe { self.dev.get_image_memory_requirements(image) };

        let alloc_info = MemoryAllocateInfo {
            allocation_size: mem_requirements.size,
            memory_type_index: self.find_memory_type(
                &self.instance,
                mem_requirements.memory_type_bits,
                properties,
            ),
            ..Default::default()
        };

        let device_memory = unsafe { self.dev.allocate_memory(&alloc_info, None).unwrap() };

        unsafe { self.dev.bind_image_memory(image, device_memory, 0).unwrap() }

        (image, device_memory)
    }

    pub fn create_image_view(
        &self,
        image: Image,
        format: Format,
        aspect_flags: ImageAspectFlags,
    ) -> ImageView {
        let view_info = ImageViewCreateInfo {
            image,
            view_type: ImageViewType::TYPE_2D,
            format,
            subresource_range: ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        return unsafe { self.dev.create_image_view(&view_info, None).unwrap() };
    }
}

pub fn create_image_views(
    device: &BigusDevice,
    swap_chain_images: Vec<Image>,
    swap_chain_image_format: Format,
) -> Vec<ImageView> {
    let mut image_views: Vec<ImageView> = Vec::with_capacity(swap_chain_images.len());

    for image in swap_chain_images {
        image_views.push(device.create_image_view(
            image,
            swap_chain_image_format,
            ImageAspectFlags::COLOR,
        ));
    }

    image_views
}

pub fn create_depth_resources(
    instance: &Instance,
    device: &BigusDevice,
    swap_chain_extent: [u32; 2],
) -> (ImageView, DeviceMemory, Format) {
    let image_format = find_depth_format(instance, &device);

    let (depth_image, depth_image_memory) = device.create_image(
        swap_chain_extent,
        image_format,
        ImageTiling::OPTIMAL,
        ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        MemoryPropertyFlags::DEVICE_LOCAL,
    );
    let depth_image_view =
        device.create_image_view(depth_image, image_format, ImageAspectFlags::DEPTH);

    (depth_image_view, depth_image_memory, image_format)
}

fn find_depth_format(instance: &Instance, device: &BigusDevice) -> Format {
    device.find_supported_format(
        instance,
        vec![
            Format::D32_SFLOAT,
            Format::D32_SFLOAT_S8_UINT,
            Format::D24_UNORM_S8_UINT,
        ],
        ImageTiling::OPTIMAL,
        FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}
