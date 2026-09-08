
use std::{any::TypeId, collections::HashMap, slice::from_raw_parts};

use crate::ecs::{component::{Component, ComponentStorage}, entity::{Entity, EntityId}};

pub struct World {
    next_entity_id: EntityId,
    components: HashMap<TypeId, Box<ComponentStorage<dyn Component>>>
}

impl World {
    pub fn new() -> Self {
        World {
            next_entity_id: 0,
            components: HashMap::new()
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.next_entity_id += 1;
        Entity {
            id: self.next_entity_id
        } 
    }

    pub fn add_component<T: Component>(&mut self,  entity: &Entity, component: T) {
        let component_bytes = unsafe { from_raw_parts((&component as *const T) as *const u8, size_of::<T>()) };

        if let Some(components) = self.components.get_mut(&T::get_type_id()) {
            components.push((entity.id, component_bytes.to_vec()));
        } else {
            self.components.insert(T::get_type_id(), vec![(entity.id, component_bytes.to_vec())]);
        }
    }
}
