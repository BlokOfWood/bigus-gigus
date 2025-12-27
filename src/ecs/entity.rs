use crate::ecs::{component::Component, ecs_runner::EcsRunner};

#[derive(Clone, Copy)]
pub struct Entity {
    pub(crate) id: u32,
}


impl Entity {
    pub fn add_component<T: Component + 'static>(&self, ecs: &mut EcsRunner, component: T) {
        ecs.add_component(self, component);
    }
}
