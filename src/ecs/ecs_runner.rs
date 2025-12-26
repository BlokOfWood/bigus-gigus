use std::slice::from_raw_parts;

use ahash::{HashMap, HashMapExt};

use crate::ecs::{
    component::{Component},
    entity::Entity,
    system::System,
};

#[allow(unused)]
pub struct EcsRunner {
    entities: Vec<Entity>,
    components: HashMap<&'static str, Vec<u8>>,
    systems: Vec<System>,
}

impl EcsRunner {
    pub fn new() -> Self {
        EcsRunner {
            entities: Vec::new(),
            components: HashMap::new(),
            systems: Vec::new(),
        }
    }

    pub fn add_component<T: Component>(&mut self, component: T) {
        println!("Adding component of type {}", T::get_type_fingerprint());

        self.components
            .entry(T::get_type_fingerprint())
            .or_insert(Vec::new())
            .extend_from_slice(unsafe {
                from_raw_parts(
                    (&component as *const T) as *const u8,
                    size_of::<T>(),
                )
            });
    }

    pub fn query<T: Component>(&self) -> Vec<&T> {
        self.components
            .get(T::get_type_fingerprint())
            .or(Some(&Vec::new()))
            .unwrap()
            .chunks(size_of::<T>())
            .map(|chunk| unsafe { &*(chunk.as_ptr() as *const T) })
            .collect()
    }

    pub fn tick(&self) {
        for system in &self.systems {
            system.tick();
        }
    }
}
