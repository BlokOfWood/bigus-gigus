use ash::vk::{
    Buffer, BufferCreateInfo, BufferImageCopy, BufferUsageFlags, CommandPool, DeviceMemory,
    Extent3D, Format, Image, ImageAspectFlags, ImageLayout, ImageSubresourceLayers,
    MemoryAllocateInfo, MemoryPropertyFlags, Offset3D, SharingMode,
    VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
};

use crate::
    vulkan::device_and_queues::BigusDevice
;

use std::mem::offset_of;

#[repr(C)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    tex_coord: [f32; 2],
}

#[repr(transparent)]
pub struct VertexData {
    pub vertices: [Vertex; 8],
}

pub const VERTICES: [Vertex; 8] = [
    Vertex {
        pos: [-0.5, -0.5, 0.0],
        color: [1.0, 0.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.0],
        color: [0.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5, 0.0],
        color: [1.0, 1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, -0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5, -0.5],
        color: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
];

pub const INDICES: [u16; 12] = [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

pub struct VertexBuffer;

impl VertexBuffer {
    pub fn get_binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Vertex>() as u32,
            input_rate: VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> [(u32, VertexInputAttributeDescription); 3] {
        [
            (
                0,
                VertexInputAttributeDescription {
                    binding: 0,
                    format: Format::R32G32B32_SFLOAT,
                    offset: offset_of!(Vertex, pos) as u32,
                    location: 0,
                },
            ),
            (
                1,
                VertexInputAttributeDescription {
                    binding: 0,
                    format: Format::R32G32B32_SFLOAT,
                    offset: offset_of!(Vertex, color) as u32,
                    location: 1,
                },
            ),
            (
                2,
                VertexInputAttributeDescription {
                    binding: 0,
                    format: Format::R32G32_SFLOAT,
                    offset: offset_of!(Vertex, tex_coord) as u32,
                    location: 2,
                },
            ),
        ]
    }
}

impl BigusDevice {
    pub fn create_buffer(
        &self,
        size: u64,
        usage: BufferUsageFlags,
        properties: MemoryPropertyFlags,
    ) -> (Buffer, DeviceMemory) {
        let buffer = unsafe {
            self.dev
                .create_buffer(
                    &BufferCreateInfo {
                        size,
                        usage,
                        sharing_mode: SharingMode::EXCLUSIVE,
                        ..Default::default()
                    },
                    None,
                )
                .unwrap()
        };

        let mem_requirements = unsafe { self.dev.get_buffer_memory_requirements(buffer) };

        let buffer_memory = unsafe {
            self.dev
                .allocate_memory(
                    &MemoryAllocateInfo {
                        memory_type_index: self.find_memory_type(
                            &self.instance,
                            mem_requirements.memory_type_bits,
                            properties,
                        ),
                        allocation_size: mem_requirements.size,
                        ..Default::default()
                    },
                    None,
                )
                .unwrap()
        };

        unsafe {
            self.dev
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .unwrap();
        };

        (buffer, buffer_memory)
    }

    pub fn copy_buffer_to_image(
        &self,
        buffer: Buffer,
        image: Image,
        command_pool: &CommandPool,
        width: u32,
        height: u32,
    ) {
        let command_buffer = self.begin_single_time_commands(command_pool);

        unsafe {
            self.dev.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                ImageLayout::TRANSFER_DST_OPTIMAL,
                &[BufferImageCopy {
                    buffer_offset: 0,
                    buffer_row_length: 0,
                    buffer_image_height: 0,
                    image_subresource: ImageSubresourceLayers {
                        aspect_mask: ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image_offset: Offset3D { x: 0, y: 0, z: 0 },
                    image_extent: Extent3D {
                        width,
                        height,
                        depth: 1,
                    },
                    ..Default::default()
                }],
            );
        };

        self.end_single_time_commands(command_pool, command_buffer);
    }
}
/*
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
        DeviceLayout::from_size_alignment(
            (std::mem::size_of::<u16>() * INDICES.len()) as u64,
            DeviceAlignment::MIN.into()
        ).unwrap(),
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
*/
