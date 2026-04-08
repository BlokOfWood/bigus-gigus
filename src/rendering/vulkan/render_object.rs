use ash::vk::{Buffer, CommandPool};

use super::{buffers::Vertex, device_and_queues::BigusDevice};
use crate::math::vector::Vector3;

pub struct RenderObject {
    pub(super) position: Vector3,
    pub(super) vertex_buffer: Buffer,
    pub(super) index_buffer: Buffer,
    pub(super) index_count: u32,
}

impl BigusDevice {
    pub(super) fn create_render_object(
        &self,
        position: Vector3,
        vertices: &[Vertex],
        indices: &[u32],
        command_pool: &CommandPool,
    ) -> RenderObject {
        let (vertex_buffer, _vertex_buffer_memory) =
            self.create_vertex_buffer(&vertices, &command_pool);

        let (index_buffer, _index_buffer_memory) =
            self.create_index_buffer(&indices, &command_pool);

        RenderObject {
            position,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}
