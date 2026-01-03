use std::any::Any;

use ahash::{HashMap, HashMapExt};

use crate::{
    ecs::{component::Component, entity::Entity, system::System},
    vulkan::vulkan::VulkanRenderer,
};

pub enum EventType {
    OnComponentCreation,
}

type EntityId = u32;
type ComponentType = &'static str;
type ComponentIndex = usize;
type CallbackIndex = usize;
type PendingCallback = dyn FnOnce(&mut VulkanRenderer) -> ();

#[allow(unused)]
pub struct EcsRunner {
    entities: Vec<Entity>,
    components: HashMap<ComponentType, Box<dyn Any>>,
    systems: Vec<System>,

    next_entity_id: EntityId,

    on_component_creation_callbacks: HashMap<ComponentType, Box<dyn Any>>,
    pending_callbacks: Vec<Box<PendingCallback>>,

    entity_component_map: HashMap<EntityId, HashMap<ComponentType, ComponentIndex>>,

    component_type_id_mappings: HashMap<ComponentType, u8>,
}

type ComponentCallback<T> = fn(T, *mut VulkanRenderer) -> ();

impl EcsRunner {
    pub fn new() -> Self {
        EcsRunner {
            entities: Vec::new(),
            components: HashMap::new(),
            systems: Vec::new(),

            next_entity_id: 0,

            on_component_creation_callbacks: HashMap::new(),
            pending_callbacks: Vec::new(),

            entity_component_map: HashMap::new(),

            component_type_id_mappings: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity {
            id: self.next_entity_id,
        };
        self.next_entity_id += 1;
        self.entities.push(entity);

        self.entities[self.entities.len() - 1]
    }

    pub fn add_on_component_creation_event<T: Component + 'static>(
        &mut self,
        callback: ComponentCallback<T>,
    ) {
        let component_type_id = T::get_type_fingerprint();

        let vec = self
            .on_component_creation_callbacks
            .entry(component_type_id)
            .or_insert(Box::new(Vec::<ComponentCallback<T>>::new()))
            .downcast_mut::<Vec<ComponentCallback<T>>>()
            .unwrap();

        vec.push(callback);
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        let component_type = T::get_type_fingerprint();

        let components_vec = match self.components.get_mut(component_type) {
            Some(components_vec_box) => components_vec_box,
            None => {
                self.component_type_id_mappings.insert(
                    T::get_type_fingerprint(),
                    self.component_type_id_mappings.len() as u8,
                );

                let new_component_vec = Box::new(Vec::<T>::new());

                self.components
                    .insert(T::get_type_fingerprint(), new_component_vec);

                self.components.get_mut(T::get_type_fingerprint()).unwrap()
            }
        }
        .downcast_mut::<Vec<T>>()
        .unwrap();

        components_vec.push(component.clone());

        self.entity_component_map
            .entry(entity.id)
            .or_insert(HashMap::new())
            .insert(T::get_type_fingerprint(), components_vec.len() - 1);

        if let Some(callbacks_box) = self.on_component_creation_callbacks.get(component_type) {
            let callbacks_vec= callbacks_box
                .downcast_ref::<Vec<ComponentCallback<T>>>()
                .unwrap();

            for callback in callbacks_vec {
                let c = component.clone();
                let cb = callback.clone();
                let pending_callback = move |renderer: &mut VulkanRenderer| {
                    cb(c, renderer);
                };
                self.pending_callbacks.push(Box::new(pending_callback));
            }
        }
    }

    pub fn resolve_events(&mut self, vulkan_renderer: &mut VulkanRenderer) {
        while let Some(callback) = self.pending_callbacks.pop()
        {
            callback(vulkan_renderer);
        }
    }

    pub fn tick(&self) {
        for system in &self.systems {
            system.tick();
        }
    }

    pub fn query<Q: Query>(&self) -> Vec<Q::Output> {
        Q::query(self)
    }
}

pub trait Query {
    type Output;
    fn query(ecs_runner: &EcsRunner) -> Vec<Self::Output>;
}

impl<T: Component + 'static> Query for (T,) {
    type Output = T;

    fn query(ecs_runner: &EcsRunner) -> Vec<Self::Output> {
        let components_vec = ecs_runner
            .components
            .get(T::get_type_fingerprint())
            .unwrap()
            .downcast_ref::<Vec<T>>()
            .unwrap();

        let mut output_vec = Vec::new();

        for mapping_lists in ecs_runner.entity_component_map.iter() {
            if let Some(mapping) = mapping_lists.1.get(T::get_type_fingerprint()) {
                output_vec.push(components_vec[*mapping].clone());
            }
        }

        output_vec
    }
}

impl<T1: Component + 'static, T2: Component + 'static> Query for (T1, T2) {
    type Output = (T1, T2);

    fn query(ecs_runner: &EcsRunner) -> Vec<Self::Output> {
        let components_vec_1 = ecs_runner
            .components
            .get(T1::get_type_fingerprint())
            .unwrap()
            .downcast_ref::<Vec<T1>>()
            .unwrap();

        let components_vec_2 = ecs_runner
            .components
            .get(T2::get_type_fingerprint())
            .unwrap()
            .downcast_ref::<Vec<T2>>()
            .unwrap();

        let mut output_vec = Vec::new();

        for mapping_lists in ecs_runner.entity_component_map.iter() {
            if let Some(mapping1) = mapping_lists.1.get(T1::get_type_fingerprint())
                && let Some(mapping2) = mapping_lists.1.get(T2::get_type_fingerprint())
            {
                output_vec.push((
                    components_vec_1[*mapping1].clone(),
                    components_vec_2[*mapping2].clone(),
                ));
            }
        }

        output_vec
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::builtins::{mesh::Mesh, position::Position};

    use super::*;

    fn setup_ecs() -> EcsRunner {
        let mut ecs = EcsRunner::new();

        let entity1 = ecs.create_entity();
        let entity2 = ecs.create_entity();
        let entity3 = ecs.create_entity();

        ecs.add_component(&entity2, Mesh::EMPTY);
        ecs.add_component(&entity1, Position::default());

        ecs.add_component(&entity3, Mesh::EMPTY);
        ecs.add_component(&entity3, Position::default());

        ecs
    }

    #[test]
    fn test_query_for_one_component() {
        let ecs = setup_ecs();

        assert_eq!(ecs.query::<(Mesh,)>().len(), 2);
        assert_eq!(ecs.query::<(Position,)>().len(), 2);
    }

    #[test]
    fn test_query_for_two_components() {
        let ecs = setup_ecs();
        assert_eq!(ecs.query::<(Mesh, Position)>().len(), 1);
    }
}
