use std::any::TypeId;

use crate::ecs::component::Component;

pub(super) trait Query {
    fn get_type_ids(&self) -> Vec<TypeId>;
}

impl<A:Component> Query for A {
    fn get_type_ids(&self) -> Vec<TypeId> {
        vec![self.get_type_id()]
    }
}

impl<A:Component, B:Component> Query for (A,B) {
    fn get_type_ids(&self) -> Vec<TypeId> {
        vec![self.0.get_type_id(), self.1.get_type_id()]
    }
}

pub(super) trait IntoSystem<Q: Query, F: Fn(Q)> {
    
}

