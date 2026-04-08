use std::any::Any;

pub mod event_handler;
pub mod redraw_requested_event;
pub mod keyboard_input_event;
pub mod resized_event;

pub trait Event: Any {}
