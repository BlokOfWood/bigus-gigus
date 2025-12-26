use component_derive::Component;

use crate::vulkan::buffers::Vertex;
use crate::ecs::component::Component;

#[derive(Component)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>, 
}