
use std::{any::TypeId, cell::RefCell, collections::HashMap, slice::from_raw_parts, sync::Arc};

use crate::{ecs::{component::Component, entity::{Entity, EntityId}, events::ComponentAddedEvent}, event::event_handler::EventHandler};

pub struct World {
    next_entity_id: EntityId,
    components: HashMap<TypeId, Vec<(EntityId, Vec<u8>)>>,
    event_handler: Arc<RefCell<EventHandler>>
}

impl World {
    pub fn new(event_handler: Arc<RefCell<EventHandler>>) -> Self {
        World {
            next_entity_id: 0,
            components: HashMap::new(),
            event_handler
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.next_entity_id += 1;
        Entity {
            id: self.next_entity_id
        } 
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        let component_bytes = unsafe { from_raw_parts((&component as *const T) as *const u8, size_of::<T>()) };

        if let Some(components) = self.components.get_mut(&T::get_type_id()) {
            components.push((entity.id, component_bytes.to_vec()));
        } else {
            self.components.insert(T::get_type_id(), vec![(entity.id, component_bytes.to_vec())]);
        }

        self.event_handler.borrow_mut().raise_event(ComponentAddedEvent { component: component.clone() });
    }
}
