use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo},
    device::{Device, Queue},
    format::Format,
    image::{sampler::ComponentMapping, Image, ImageCreateInfo, ImageLayout, ImageTiling, ImageType, ImageUsage, SampleCount},
};
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    image::sampler::{
        BorderColor, Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode,
    },
    sync::{GpuFuture, Sharing},
};
use vulkano::{
    image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        ImageAspects, ImageSubresourceRange,
    },
    memory::{
        allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
        MemoryPropertyFlags,
    },
    sync::{self},
};

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

pub fn create_image_views(
    swap_chain_images: &Vec<Arc<Image>>,
    image_format: Format,
) -> Vec<Arc<ImageView>> {
    let mut image_views: Vec<Arc<ImageView>> = Vec::with_capacity(swap_chain_images.len());

    for image in swap_chain_images {
        let create_info = ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            format: image_format,
            component_mapping: ComponentMapping::identity(),
            subresource_range: ImageSubresourceRange {
                array_layers: 0..1,
                aspects: ImageAspects::COLOR,
                mip_levels: 0..1,
            },
            ..Default::default()
        };

        let image_view = ImageView::new(image.clone(), create_info);
        image_views.push(image_view.unwrap());
    }

    image_views
}
