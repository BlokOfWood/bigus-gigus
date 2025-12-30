use ahash::{HashMap, HashMapExt};

use crate::resource_handling::{
    loadable::LoadableResource,
    resource::Resource,
    resource_errors::{ResourceLoadError, ResourceReferenceError},
    resource_handle::ResourceHandle,
};

/**
   The resource handler has the following goals:
    * Make resource loading its own responsibility
    * Abstract away resource ownership
*/
pub struct ResourceHandler {
    resources: HashMap<u32, Box<dyn Resource>>,
    next_id: u32,
}

impl ResourceHandler {
    pub fn new() -> Self {
        ResourceHandler {
            resources: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn retrieve_resource<T: Resource + 'static>(
        &self,
        resource_handle: &ResourceHandle<T>,
    ) -> Result<&T, ResourceReferenceError> {
        if !resource_handle.is_valid {
            return Err(ResourceReferenceError::InvalidResourceHandle);
        }

        if let Some(resource) = &self.resources.get(&resource_handle.id) {
            let requested_resource = resource.as_any().downcast_ref::<T>();

            match requested_resource {
                Some(res) => Ok(res),
                None => Err(ResourceReferenceError::TypeMismatch),
            }
        } else {
            Err(ResourceReferenceError::InvalidResourceHandle)
        }
    }

    pub fn load_resource<T: LoadableResource>(
        &mut self,
        path: &str,
    ) -> Result<ResourceHandle<T>, ResourceLoadError> {
        let resource_load_result = T::load(path);

        match resource_load_result {
            Ok(resource) => Ok(self.insert_resource(resource)),
            Err(err) => Err(err),
        }
    }

    // Inserts a resource into the resource handler and returns its id.
    fn insert_resource<T: Resource + 'static>(&mut self, resource: T) -> ResourceHandle<T> {
        self.next_id += 1;

        self.resources.insert(self.next_id, Box::new(resource));

        return ResourceHandle::new(self.next_id);
    }

    pub fn release_resource<T: Resource>(&mut self, mut resource_handle: ResourceHandle<T>) {
        self.resources.remove(&resource_handle.id);
        resource_handle.is_valid = false;
    }
}
