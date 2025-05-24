use ash::{
    vk::{
        DeviceMemory, Extent3D, Format, FormatFeatureFlags, Image, ImageAspectFlags,
        ImageCreateInfo, ImageLayout, ImageSubresourceRange, ImageTiling, ImageType,
        ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, MemoryAllocateInfo,
        MemoryPropertyFlags, SampleCountFlags, SharingMode,
    },
    Instance,
};

use super::device_and_queues::BigusDevice;

/*
pub fn create_texture_image(
    allocator: Arc<dyn MemoryAllocator>,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: Arc<Queue>,
    device: Arc<Device>,
) -> Arc<Image> {
    let open_image = image::open("assets/textures/statue.jpg").unwrap();

    let image_extent = [open_image.width(), open_image.height(), 1];

    let image_data = open_image.to_rgba8();

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let staging_buffer: Subbuffer<[u8]> = Buffer::from_iter(
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
        image_data.into_raw(),
    )
    .unwrap();

    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            extent: image_extent,
            mip_levels: 1,
            array_layers: 1,
            format: Format::R8G8B8A8_SRGB,
            tiling: ImageTiling::Optimal,
            initial_layout: ImageLayout::Undefined,
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            sharing: Sharing::Exclusive,
            samples: SampleCount::Sample1,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter {
                required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();

    command_buffer_builder
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            staging_buffer,
            image.clone(),
        ))
        .unwrap();

    let command_buffer = command_buffer_builder.build().unwrap();

    let mut now = sync::now(device.clone());
    now.cleanup_finished();
    let gpu_future = now.boxed();

    let _ = gpu_future
        .then_execute(queue, command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None);

    image
}
pub fn create_texture_image_view(image: Arc<Image>) -> Arc<ImageView> {
    ImageView::new(
        image,
        ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            format: Format::R8G8B8A8_SRGB,
            subresource_range: ImageSubresourceRange {
                aspects: ImageAspects::COLOR,
                mip_levels: 0..1,
                array_layers: 0..1,
            },
            ..Default::default()
        },
    )
    .unwrap()
}

pub fn create_texture_sampler(device: Arc<Device>) -> Arc<Sampler> {
    Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            address_mode: [SamplerAddressMode::Repeat; 3],
            anisotropy: Some(device.physical_device().properties().max_sampler_anisotropy),
            border_color: BorderColor::IntOpaqueBlack,
            unnormalized_coordinates: false,
            compare: None,
            mipmap_mode: SamplerMipmapMode::Linear,
            mip_lod_bias: 0.0,
            lod: 0.0..=0.0,
            ..Default::default()
        },
    )
    .unwrap()
}
*/
impl BigusDevice {
    pub fn create_image(
        &self,
        instance: &Instance,
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
                instance,
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

pub fn create_depth_resources(instance: &Instance, device: &BigusDevice, swap_chain_extent: [u32; 2]) -> (ImageView, DeviceMemory, Format) {
    let image_format = find_depth_format(instance, &device);

    let (depth_image, depth_image_memory) = device.create_image(instance, swap_chain_extent, image_format, ImageTiling::OPTIMAL, ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, MemoryPropertyFlags::DEVICE_LOCAL);
    let depth_image_view = device.create_image_view(depth_image, image_format, ImageAspectFlags::DEPTH);

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
