use std::any::TypeId;

use crate::ecs::entity::EntityId;

pub(super) type ComponentId = u64;

pub trait Component: Clone + Sized {
    fn get_type_id(self: &Self) -> TypeId;
}

pub(super) struct ComponentInstance {
    pub(super) entity_id: EntityId,
    pub(super) component_id: ComponentId,
    pub(super) component_bytes: Vec<u8>,
}

impl ComponentInstance {
    pub(super) fn new(entity_id: EntityId, component_id: ComponentId, component_bytes: Vec<u8>) -> Self {
        ComponentInstance {
            entity_id,
            component_id,
            component_bytes
        }
    }
}
