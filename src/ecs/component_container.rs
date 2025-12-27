use std::{any::{Any, TypeId}, collections::HashMap};

use crate::ecs::{component::Component, entity::Entity};

pub struct ComponentContainer {
    components: HashMap<TypeId, Box<dyn Any>>,
    entity_mappings: HashMap<u32, Vec<(TypeId, usize)>>,
}

impl ComponentContainer {
    pub fn new() -> Self {
        ComponentContainer {
            components: HashMap::new(),
            entity_mappings: HashMap::new(),
        }
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        let component_type_id = component.type_id();

        let vec = self.components
            .entry(component_type_id)
            .or_insert(Box::new(Vec::<T>::new()))
            .downcast_mut::<Vec<T>>()
            .unwrap();

        vec.push(component);

        self.entity_mappings
            .entry(entity.id)
            .or_insert(Vec::new())
            .push((component_type_id, vec.len() - 1));
    }

    pub fn query<T: Component + 'static>(&self) -> Vec<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|boxed_vec| boxed_vec.downcast_ref::<Vec<T>>())
            .map(|vec| vec.iter().collect())
            .unwrap_or_else(Vec::new)
    }
}
