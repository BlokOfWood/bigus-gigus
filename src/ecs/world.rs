
use std::{any::TypeId, cell::RefCell, collections::HashMap, marker::PhantomData, slice::from_raw_parts, sync::Arc};

use crate::{ecs::{builtins::transform::Transform, component::{Component, ComponentId, ComponentInstance}, entity::{Entity, EntityId}, events::ComponentAddedEvent, system::{IntoSystem, Query}}, event::event_handler::EventHandler};

pub struct World {
    next_entity_id: EntityId,
    next_component_id: ComponentId,
    entities: HashMap<EntityId, Vec<(TypeId, ComponentId)>>,
    archetypes: Vec<(Vec<TypeId>, Vec<Vec<Box<dyn Component>>>)>,
    systems: Vec<Box<dyn Fn(Box<dyn Query>)>>,
    event_handler: Arc<RefCell<EventHandler>>
}

impl World {
    pub fn new(event_handler: Arc<RefCell<EventHandler>>) -> Self {
        World {
            next_entity_id: 0,
            next_component_id: 0,
            entities: HashMap::new(),
            archetypes: Vec::new(),
            systems: Vec::new(),
            event_handler
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.next_entity_id += 1;

        let new_entity = Entity {
            id: self.next_entity_id
        }; 
        
        self.entities.insert(new_entity.id, Vec::new());

        self.add_component(&new_entity, Transform::default());

        new_entity
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        let component_bytes = unsafe { from_raw_parts((&component as *const T) as *const u8, size_of::<T>()) };

        self.next_component_id += 1;
        let new_component = ComponentInstance::new(entity.id, self.next_component_id, component_bytes.to_vec());

        self.entities.entry(entity.id)
            .or_default()
            .push((component.get_type_id(), new_component.component_id));

        /*self.components.entry(component.get_type_id())
            .or_default()
            .push(new_component);*/

        self.event_handler.borrow_mut().raise_event(ComponentAddedEvent { component: component.clone() });
    }

    pub fn tick(&mut self) {
        for system in &self.systems {
           system(Box::new(Transform::default())); 
        }
    }

    pub fn add_system<F: Fn(Box<dyn Query>) + 'static>(&mut self, system: F) {
        self.systems.push(Box::new(system));
    }
}

