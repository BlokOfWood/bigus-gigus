use std::any::Any;

use ahash::{HashMap, HashMapExt};

use crate::{app::App, ecs::{component::Component, entity::Entity}, rendering::vulkan::VulkanRenderer};

pub enum EventType {
    OnComponentCreation,
}

pub mod bit_set;
pub mod builtins;
pub mod component;
pub mod entity;
pub mod plugin;
pub mod singleton_handler;
pub mod system;
pub mod world;

