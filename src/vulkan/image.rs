use ash::vk::{
    Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateInfo,
    ImageViewType,
};

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

pub fn create_image_view(
    device: ash::Device,
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

    return unsafe { device.create_image_view(&view_info, None).unwrap() };
}

pub fn create_image_views(
    device: ash::Device,
    swap_chain_images: Vec<Image>,
    swap_chain_image_format: Format,
) -> Vec<ImageView> {
    let mut image_views: Vec<ImageView> = Vec::with_capacity(swap_chain_images.len());

    for image in swap_chain_images {
        image_views.push(create_image_view(
            device.clone(),
            image,
            swap_chain_image_format,
            ImageAspectFlags::COLOR,
        ));
    }

    image_views
}

/*
pub fn create_depth_resources(
    device: BigusDevice,
    allocator: Arc<dyn MemoryAllocator>,
    image_extent: [u32; 3],
) -> (Arc<ImageView>, Format) {
    let image_format = device.find_supported_format(
        vec![
            Format::D32_SFLOAT,
            Format::D32_SFLOAT_S8_UINT,
            Format::D24_UNORM_S8_UINT,
        ],
        ImageTiling::Optimal,
        FormatFeatures::DEPTH_STENCIL_ATTACHMENT,
    );

    //let has_stencil_component =
    //    image_format == Format::D32_SFLOAT_S8_UINT || image_format == Format::D24_UNORM_S8_UINT;

    let depth_image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            extent: image_extent,
            mip_levels: 1,
            array_layers: 1,
            format: image_format,
            tiling: ImageTiling::Optimal,
            initial_layout: ImageLayout::Undefined,
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
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

    let depth_image_view = ImageView::new(
        depth_image.clone(),
        ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            format: image_format,
            component_mapping: ComponentMapping::identity(),
            subresource_range: ImageSubresourceRange {
                array_layers: 0..1,
                aspects: ImageAspects::DEPTH,
                mip_levels: 0..1,
            },
            ..Default::default()
        },
    ).unwrap();

    (depth_image_view, image_format)
}
*/
