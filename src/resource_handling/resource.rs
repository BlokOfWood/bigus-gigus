use std::any::Any;

use crate::resource_handling::resource_type::ResourceType;

pub trait Resource {
    fn resource_type(&self) -> ResourceType;
    fn as_any(&self) -> &dyn Any;
}