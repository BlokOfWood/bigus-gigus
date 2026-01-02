use crate::ecs::component::Component;
use component_derive::Component;

use crate::math::vector::Vector3;

#[derive(Component, Clone)]
pub struct Position {
    pub position: Vector3,
}

impl Default for Position {
    fn default() -> Self {
        Self { position: Default::default() }
    }
}