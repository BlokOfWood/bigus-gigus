use std::{any::{Any, TypeId}, collections::HashMap};

use crate::event::Event;

pub struct EventHandler {
    handlers: HashMap<TypeId, Vec<Box<dyn Fn(&Box<dyn Any>)>>>,
    pending_events: Vec<(TypeId, Box<dyn Any>)>
}


impl EventHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            pending_events: Vec::new()
        }
    }

    pub fn add_handler<E: Event, F: Fn(&E) -> () + 'static>(&mut self, handler: F) {
        let new_handler = move |event: &Box<dyn Any>| {
            let event = event.downcast_ref().unwrap();
            handler(event);
        };

        self.handlers
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(new_handler));
    }

    pub fn raise_event<T: Event>(&mut self, event: T) {
        self.pending_events.push((event.type_id(), Box::new(event)));
    }

    pub(crate) fn resolve_events(&mut self) {
        for event in &mut self.pending_events {
            for handler in &self.handlers[&event.0] {
                handler(&event.1);
            }
        }
        self.pending_events.clear();
    }
}
