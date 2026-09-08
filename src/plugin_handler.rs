use std::{any::TypeId, collections::HashMap};

use crate::ecs::plugin::Plugin;

pub struct PluginHandler {
    plugins: HashMap<TypeId, Box<dyn Plugin>>,
}

impl PluginHandler {
    pub fn new() -> Self {
        PluginHandler {
            plugins: HashMap::new(),
        }
    }

    pub(crate) fn add_plugin<T: Plugin + 'static>(&mut self, plugin: T) {
        self.plugins.insert(TypeId::of::<T>(), Box::new(plugin));
    }
}
