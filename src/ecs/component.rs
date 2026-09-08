use std::any::TypeId;

use crate::ecs::entity::{Entity, EntityId};

pub trait Component: Clone + Sized {
    fn get_type_id() -> TypeId;
}

#[derive(Clone)]
pub(super) struct ComponentStorage<T: Component> {
    storage: Vec<(EntityId, T)> 
} 

impl<T: Component> ComponentStorage<T> {
    pub(super) fn new() -> Self {
        Self {
            storage: Vec::new()
        }
    }

    pub(super) fn add_item(&mut self, entity: Entity, component: T) {
        self.storage.push((entity.id, component));
    }
}
