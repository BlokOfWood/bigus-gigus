use crate::ecs::Query;

pub trait System<T: Query> {
    fn tick(query: T) -> ();
}