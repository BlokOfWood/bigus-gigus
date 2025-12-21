use crate::resource_handling::{resource::Resource, resource_errors::ResourceLoadError};

pub trait Loadable: Resource + Sized + 'static {
    fn load(path: &str) -> Result<Self, ResourceLoadError>;
}