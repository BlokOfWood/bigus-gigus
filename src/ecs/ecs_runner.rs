use crate::ecs::{component::Component, entity::Entity, system::System};

pub struct EcsRunner {
    entities: Vec<Entity>,
    components: Vec<Component>,
    systems: Vec<System>,
}

impl EcsRunner {
    pub fn new() -> Self {
        EcsRunner {
            entities: Vec::new(),
            components: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn tick(&self) {
        for system in &self.systems {
            system.tick();
        }
    }
}
