use std::fs::read_to_string;

use crate::resource_handler::{loadable::LoadableResource, resource::Resource, resource_errors::ResourceLoadError};

pub struct TextResource {
   pub text: String 
}

impl Resource for TextResource {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LoadableResource for TextResource {
    fn load(path: &str) -> Result<Self, ResourceLoadError> {
        match read_to_string(path) {
            Ok(content) => Ok(TextResource { text: content }),
            Err(_) => Err(ResourceLoadError::UnableToLoad),
        }
    }
}