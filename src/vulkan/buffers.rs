use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    device::{Device, Queue},
    memory::{
        allocator::{AllocationCreateInfo, DeviceLayout, MemoryAllocator, MemoryTypeFilter},
        DeviceAlignment, MemoryPropertyFlags,
    },
    sync::{self, GpuFuture, Sharing},
    Validated, VulkanError,
};

use crate::math::{graphics_ops::{look_at, perspective}, quaternion::Quaternion, vector::{Vector3, VECTOR3_ZERO}};

use super::{
    command_pool::CommandPool,
    ubo::UniformBufferObject,
    vertex_buffer::{VertexData, INDICES, VERTICES},
    vulkan::VulkanRenderer,
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

pub(super) fn create_uniform_buffers(
    max_frames_in_flight: usize,
    allocator: Arc<dyn MemoryAllocator>,
) -> Result<Vec<Subbuffer<UniformBufferObject>>, Validated<VulkanError>> {
    let mut uniform_buffers: Vec<Subbuffer<UniformBufferObject>> =
        Vec::with_capacity(max_frames_in_flight);

    // TODO: convert to persistent mapping
    for _ in 0..max_frames_in_flight {
        let uniform_buffer = Buffer::new_sized::<UniformBufferObject>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
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
        )
        .unwrap();

        uniform_buffers.push(uniform_buffer.clone());
    }

    return Ok(uniform_buffers);
}

impl VulkanRenderer {
    pub(super) fn update_uniform_buffer(&mut self, image_index: usize, aspect_ratio: f32) {
        let elapsed_time = self.start_time.elapsed();

        let mut uniform_buffer = self.uniform_buffers[image_index].write().unwrap();

        uniform_buffer.model =
            Quaternion::new(Vector3::new(0.0, 0.0, 1.0), elapsed_time.as_secs_f32())
                .into_rotation_matrix();

        uniform_buffer.view = look_at(
            VECTOR3_ZERO,
            Vector3::new(0.0, elapsed_time.as_secs_f32().sin() * 5.0, 2.0),
        );
        uniform_buffer.proj = perspective(60.0, aspect_ratio, 0.1, 100.0);
    }
}
