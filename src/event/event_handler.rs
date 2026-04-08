use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::event::Event;

pub struct EventHandler {
    pending_events: Vec<(TypeId, Box<dyn Event>)>,
    callbacks: HashMap<TypeId, Vec<Box<dyn FnMut(&dyn Any) -> ()>>>,
}
impl EventHandler {
    pub fn new() -> Self {
        EventHandler {
            pending_events: Vec::new(),
            callbacks: HashMap::new(),
        }
    }

    pub fn raise_event<T: Event + 'static>(&mut self, event: T) {
        self.pending_events
            .push((TypeId::of::<T>(), Box::new(event)));
    }

    pub fn add_callback<T: Event + 'static, U: FnMut(&T) -> () + 'static>(
        &mut self,
        mut callback: U,
    ) {
        let wrapped: Box<dyn FnMut(&dyn Any)> = Box::new(move |event_any: &dyn Any| {
            if let Some(event) = event_any.downcast_ref::<T>() {
                callback(event);
            }
        });

        self.callbacks
            .entry(TypeId::of::<T>())
            .or_insert(Vec::new())
            .push(wrapped);
    }

    pub fn resolve_events(&mut self) {
        for (type_id, event) in &self.pending_events {
            if let Some(callbacks) = self.callbacks.get_mut(type_id) {
                let event_any = event.as_ref() as &dyn Any;
                for callback in callbacks {
                    callback(event_any);
                }
            }
        }

        self.pending_events.clear();
    }
}
