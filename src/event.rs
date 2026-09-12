use std::any::Any;

pub mod event_handler;

pub trait Event: 'static + Any {}

impl<T> Event for T where T: 'static {}
