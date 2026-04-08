use component_derive::Component;

use crate::ecs::component::Component;
use crate::resource_handler::resources::mesh::Model;
use crate::rendering::vulkan::buffers::Vertex;

#[derive(Component, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub const EMPTY: Self = Self {
        indices: Vec::new(),
        vertices: Vec::new(),
    };

    pub fn from_model_resource(model: &Model) -> Self {
        Mesh {
            vertices: model.vertices.clone(),
            indices: model.indices.clone(),
        }
    }
}
