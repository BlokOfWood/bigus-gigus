use std::mem::offset_of;

use vulkano::{
    buffer::BufferContents,
    format::Format,
    pipeline::graphics::vertex_input::{
        VertexInputAttributeDescription, VertexInputBindingDescription,
    },
};

#[derive(BufferContents)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

#[derive(BufferContents)]
#[repr(transparent)]
pub struct VertexData {
    pub vertices: [Vertex; 3],
}

pub const VERTICES: [Vertex; 3] = [
    Vertex {
        pos: [0.0, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [1.0, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-1.0, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

pub struct VertexBuffer;

impl VertexBuffer {
    pub fn get_binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            stride: size_of::<Vertex>() as u32,
            input_rate: vulkano::pipeline::graphics::vertex_input::VertexInputRate::Vertex,
        }
    }

    pub fn get_attribute_descriptions() -> [(u32, VertexInputAttributeDescription); 2] {
        [
            (
                0,
                VertexInputAttributeDescription {
                    binding: 0,
                    format: Format::R32G32_SFLOAT,
                    offset: offset_of!(Vertex, pos) as u32,
                },
            ),
            (
                1,
                VertexInputAttributeDescription {
                    binding: 0,
                    format: Format::R32G32B32_SFLOAT,
                    offset: offset_of!(Vertex, color) as u32,
                },
            ),
        ]
    }
}
