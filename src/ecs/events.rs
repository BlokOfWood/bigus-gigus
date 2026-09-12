use crate::ecs::component::Component;

pub struct ComponentAddedEvent<T: Component> {
    pub component: T
}

