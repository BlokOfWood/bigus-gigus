use std::any::Any;

pub trait Resource {
    fn as_any(&self) -> &dyn Any;
}