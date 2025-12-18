use ash::vk::{
    Buffer, BufferCopy, BufferCreateInfo, BufferImageCopy, BufferUsageFlags, CommandPool,
    DeviceMemory, Extent3D, Format, Image, ImageAspectFlags, ImageLayout, ImageSubresourceLayers,
    MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, Offset3D, SharingMode,
    VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
};

use crate::{
    math::{
        graphics_ops::{look_at, perspective},
        quaternion::Quaternion,
        vector::{Vector3, VECTOR3_ZERO},
    },
    vulkan::{
        device_and_queues::BigusDevice,
        ubo::UniformBufferObject,
        vulkan::{VulkanRenderer, MAX_FRAMES_IN_FLIGHT},
    },
};

use std::{mem::offset_of, os::raw::c_void};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

/*pub const VERTICES: [Vertex; 8] = [
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

pub const INDICES: [u32; 12] = [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];*/

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

    fn copy_buffer(
        &self,
        command_pool: &CommandPool,
        src_buffer: Buffer,
        dst_buffer: Buffer,
        size: u64,
    ) {
        let command_buffer = self.begin_single_time_commands(command_pool);

        unsafe {
            self.dev.cmd_copy_buffer(
                command_buffer,
                src_buffer,
                dst_buffer,
                &[BufferCopy {
                    size,
                    ..Default::default()
                }],
            )
        };

        self.end_single_time_commands(command_pool, command_buffer);
    }

    pub fn copy_into_buffer<T>(&self, buffer_memory: DeviceMemory, src_data: &[T]) {
        let src_data_size = src_data.len() * size_of_val(&src_data[0]);

        unsafe {
            let data = self
                .dev
                .map_memory(
                    buffer_memory,
                    0,
                    src_data_size as u64,
                    MemoryMapFlags::empty(),
                )
                .unwrap();
            data.copy_from(src_data.as_ptr() as *mut c_void, src_data_size);

            self.dev.unmap_memory(buffer_memory);
        };
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

    pub(crate) fn create_vertex_buffer(
        &self,
        vertex_array: &[Vertex], 
        command_pool: &CommandPool,
    ) -> (Buffer, DeviceMemory) {
        let buffer_size = (size_of::<Vertex>() * vertex_array.len()) as u64;

        let (staging_buffer, staging_buffer_memory) = self.create_buffer(
            buffer_size,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        );

        self.copy_into_buffer(staging_buffer_memory, &vertex_array);

        let (vertex_buffer, vertex_buffer_memory) = self.create_buffer(
            buffer_size,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        self.copy_buffer(command_pool, staging_buffer, vertex_buffer, buffer_size);

        unsafe {
            self.dev.destroy_buffer(staging_buffer, None);
            self.dev.free_memory(staging_buffer_memory, None);
        };

        (vertex_buffer, vertex_buffer_memory)
    }

    pub(crate) fn create_index_buffer(&self, index_array: &[u32], command_pool: &CommandPool) -> (Buffer, DeviceMemory) {
        let buffer_size = (size_of::<u32>() * index_array.len()) as u64;

        let (staging_buffer, staging_buffer_memory) = self.create_buffer(
            buffer_size,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        );

        self.copy_into_buffer(staging_buffer_memory, index_array);

        let (index_buffer, index_buffer_memory) = self.create_buffer(
            buffer_size,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        self.copy_buffer(command_pool, staging_buffer, index_buffer, buffer_size);

        unsafe {
            self.dev.destroy_buffer(staging_buffer, None);
            self.dev.free_memory(staging_buffer_memory, None);
        };

        (index_buffer, index_buffer_memory)
    }

    pub(crate) fn create_uniform_buffers(
        &self,
    ) -> (Vec<Buffer>, Vec<DeviceMemory>, Vec<*mut c_void>) {
        let buffer_size = size_of::<UniformBufferObject>() as u64;

        let mut uniform_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut uniform_buffer_memories = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut uniform_buffers_mapped = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (uniform_buffer, uniform_buffer_memory) = self.create_buffer(
                buffer_size,
                BufferUsageFlags::UNIFORM_BUFFER,
                MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            );
            uniform_buffers.push(uniform_buffer);
            uniform_buffer_memories.push(uniform_buffer_memory);

            uniform_buffers_mapped.push(unsafe {
                self.dev
                    .map_memory(
                        uniform_buffer_memory,
                        0,
                        buffer_size,
                        MemoryMapFlags::empty(),
                    )
                    .unwrap()
            });
        }

        (
            uniform_buffers,
            uniform_buffer_memories,
            uniform_buffers_mapped,
        )
    }
}

impl VulkanRenderer {
    pub(super) fn update_uniform_buffer(&mut self, current_frame: usize, aspect_ratio: f32) {
        let view= look_at(
            VECTOR3_ZERO,
            Vector3::new(self.x, self.y, self.z),
        );

        let uniform_buffer_new_contents = UniformBufferObject {
            model: Quaternion::new(Vector3::new(0.0 , 1.0, 0.0), 0.0/*elapsed_time.as_secs_f32()*/)
                .into_rotation_matrix(),
            view,
            proj: perspective(60.0, aspect_ratio, 0.1, 100.0),
        };

        unsafe {
            self.uniform_buffers_mapped[current_frame].copy_from(
                &uniform_buffer_new_contents as *const _ as *const c_void,
                size_of::<UniformBufferObject>(),
            );
        }
    }
}
