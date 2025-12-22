use crate::{app::App, resource_handling::{loadable::LoadableResource, resource::Resource, resource_errors::{ResourceLoadError, ResourceReferenceError}, resource_handle::ResourceHandle}};

impl App {
    pub fn retreive_resource<T: Resource + 'static>(
        &self,
        resource_handle: &ResourceHandle<T>,
    ) -> Result<&T, ResourceReferenceError> {
        self.resource_handler.retreive_resource(resource_handle)
    }

    pub fn load_resource<T: LoadableResource>(
        &mut self,
        path: &str,
    ) -> Result<ResourceHandle<T>, ResourceLoadError> {
        self.resource_handler.load_resource(path)
    }

    pub fn release_resource<T: Resource>(&mut self, resource_handle: ResourceHandle<T>) {
        self.resource_handler.release_resource(resource_handle);
    }
}