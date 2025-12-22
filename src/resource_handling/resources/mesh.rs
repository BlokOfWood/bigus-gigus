use std::{any::Any, path::Path};

use crate::{
    resource_handling::{
        loadable::LoadableResource, resource::Resource, resource_errors::ResourceLoadError, resources::mesh::obj_loader::load_obj,
    },
    vulkan::buffers::Vertex,
};

mod obj_loader;

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Resource for Model {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LoadableResource for Model {
    fn load(path: &str) -> Result<Self, ResourceLoadError> {
        let extension = match Path::new(path).extension() {
            Some(extension) if extension == "obj" => extension,
            _ => return Err(ResourceLoadError::UnknownFileType)
        };

        match extension.to_str() {
            Some("obj") => Ok(load_obj(path)),
            _ => return Err(ResourceLoadError::UnableToLoad)
        }
    }
}
