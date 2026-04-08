use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct SingletonHandler {
    singletons: HashMap<TypeId, Box<dyn Any>>,
}

impl SingletonHandler {
    pub fn new() -> Self {
        SingletonHandler {
            singletons: HashMap::new(),
        }
    }

    pub fn add_or_replace_singleton<T: 'static>(&mut self, singleton: T) {
        self.singletons
            .insert(singleton.type_id(), Box::new(singleton));
    }

    pub fn get_singleton<T: 'static>(&mut self) -> &T {
        let singleton = self.singletons.get(&TypeId::of::<T>());

        singleton.unwrap().downcast_ref().unwrap()
    }
}
