use std::marker::PhantomData;

use crate::resource_handler::resource::Resource;

pub struct ResourceHandle<T: Resource> {
    pub(super) id: u32,
    pub(super)is_valid: bool,
    phantom: PhantomData<T>
}

impl<T: Resource> ResourceHandle<T> {
    pub(crate) fn new(resource_id: u32) -> Self {
        ResourceHandle {
            id: resource_id,
            is_valid: true,
            phantom: PhantomData,
        }
    }
}

impl<T: Resource> Drop for ResourceHandle<T> {
    fn drop(&mut self) {
        self.is_valid = false;
    }
}
