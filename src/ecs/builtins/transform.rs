use crate::ecs::component::Component;
use component_derive::Component;

use crate::math::vector::Vector3;

#[derive(Component, Clone)]
pub struct Transform {
    pub position: Vector3,
}

impl Default for Transform {
    fn default() -> Self {
        Self { position: Default::default() }
    }
}
