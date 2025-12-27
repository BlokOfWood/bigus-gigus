use crate::vulkan::buffers::Vertex;
use crate::ecs::component::Component;

pub struct Mesh  {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>, 
}

impl Component for Mesh {
}