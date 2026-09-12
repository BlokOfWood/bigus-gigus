use std::any::TypeId;

pub trait Component: Clone + Sized {
    fn get_type_id() -> TypeId;
}

