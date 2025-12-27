use crate::ecs::{
    component::Component, component_container::ComponentContainer, entity::Entity, system::System,
};

#[allow(unused)]
pub struct EcsRunner {
    entities: Vec<Entity>,
    components: ComponentContainer,
    systems: Vec<System>,
    next_entity_id: u32,
}

impl EcsRunner {
    pub fn new() -> Self {
        EcsRunner {
            entities: Vec::new(),
            components: ComponentContainer::new(),
            systems: Vec::new(),
            next_entity_id: 0,
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

    pub fn add_component<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        self.components.add_component(entity, component);
    }

    pub fn query<T: Component + 'static>(&self) -> Vec<&T> {
        self.components.query::<T>()
    }

    pub fn tick(&self) {
        for system in &self.systems {
            system.tick();
        }
    }
}
