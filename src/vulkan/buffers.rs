use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::{Device, Queue},
    memory::{
        allocator::{AllocationCreateInfo, DeviceLayout, MemoryAllocator, MemoryTypeFilter},
        DeviceAlignment, MemoryPropertyFlags,
    },
    sync::{self, GpuFuture, Sharing},
};

use super::{
    command_pool::CommandPool,
    vertex_buffer::{VertexData, INDICES, VERTICES},
};

pub(crate) fn create_vertex_buffer(
    dev: Arc<Device>,
    graphics_queue: Arc<Queue>,
    command_pool: &CommandPool,
    allocator: Arc<dyn MemoryAllocator>,
) -> Arc<Buffer> {
    let staging_buffer = Buffer::from_data(
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
        VertexData { vertices: VERTICES },
    )
    .unwrap();

    let vertex_buffer = Buffer::new(
        allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX_BUFFER,
            sharing: Sharing::Exclusive,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter {
                required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
                ..Default::default()
            },
            ..Default::default()
        },
        DeviceLayout::from_size_alignment(
            (size_of::<VertexData>()) as u64,
            DeviceAlignment::MIN.into(),
        )
        .unwrap(),
    )
    .unwrap();

    let command_buffer =
        command_pool.record_copy_pass(staging_buffer.into_bytes(), vertex_buffer.clone().into());

    let mut now = sync::now(dev.clone());
    now.cleanup_finished();
    let gpu_future = now.boxed();

    let _ = gpu_future
        .then_execute(graphics_queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None);

    vertex_buffer
}

pub(crate) fn create_index_buffer(
    dev: Arc<Device>,
    graphics_queue: Arc<Queue>,
    command_pool: &CommandPool,
    allocator: Arc<dyn MemoryAllocator>,
) -> Arc<Buffer> {
    let staging_buffer = Buffer::from_data(
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
        INDICES,
    )
    .unwrap();

    let index_buffer = Buffer::new(
        allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST | BufferUsage::INDEX_BUFFER,
            sharing: Sharing::Exclusive,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter {
                required_flags: MemoryPropertyFlags::DEVICE_LOCAL,
                ..Default::default()
            },
            ..Default::default()
        },
        DeviceLayout::from_size_alignment(12, DeviceAlignment::MIN.into()).unwrap(),
    )
    .unwrap();

    let command_buffer =
        command_pool.record_copy_pass(staging_buffer.into_bytes(), index_buffer.clone().into());

    let mut now = sync::now(dev.clone());
    now.cleanup_finished();
    let gpu_future = now.boxed();

    let _ = gpu_future
        .then_execute(graphics_queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None);

    index_buffer
}
