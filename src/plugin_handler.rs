use crate::ecs::plugin::Plugin;

pub struct PluginHandler {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginHandler {
    pub fn new() -> Self {
        PluginHandler {
            plugins: Vec::new(),
        }
    }

    pub(crate) fn add_plugin<T: Plugin + 'static>(&mut self, plugin: T) {
        self.plugins.push(Box::new(plugin));
    }
}
