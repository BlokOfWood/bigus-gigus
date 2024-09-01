use std::sync::Arc;

use vulkano::{
    command_buffer::{pool::{CommandBufferAllocateInfo, CommandPoolAlloc, CommandPoolCreateFlags, CommandPoolCreateInfo}, sys::CommandBufferBeginInfo, AutoCommandBufferBuilder, CommandBufferLevel, CommandBufferUsage},
    device::{physical::PhysicalDevice, Device},
    swapchain::Surface,
};

use super::queue_family::QueueFamilyIndices;

pub struct CommandPool {
    command_pool: vulkano::command_buffer::pool::CommandPool,
    queue_family_index: u32,
}

impl CommandPool {
    pub fn new(dev: Arc<Device>, phys_dev: Arc<PhysicalDevice>, surface: Arc<Surface>) -> Self {
        let queue_families = QueueFamilyIndices::find_queue_families(phys_dev, surface);

        let command_pool_create_info = CommandPoolCreateInfo {
            queue_family_index: queue_families.graphics_family.unwrap(),
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };

        CommandPool {
            command_pool: vulkano::command_buffer::pool::CommandPool::new(
                dev,
                command_pool_create_info,
            )
            .unwrap(),
            queue_family_index: queue_families.graphics_family.unwrap()
        }
    }

    pub fn alloc_buffer(&self) -> CommandBuffer {
        let alloc_info = CommandBufferAllocateInfo {
            level: CommandBufferLevel::Primary,
            command_buffer_count: 1,
            ..Default::default()
        };

        let command_pool_alloc = self.command_pool
            .allocate_command_buffers(alloc_info)
            .unwrap()
            .last()
            .unwrap();

        AutoCommandBufferBuilder::primary(&self.command_pool, self.queue_family_index, CommandBufferUsage::MultipleSubmit)
    }
}

pub struct CommandBuffer {
    pool_alloc: CommandPoolAlloc,
}

impl CommandBuffer {
    pub fn new(command_pool_alloc: CommandPoolAlloc, family_index: u32) -> Self {

        CommandBuffer {
            pool_alloc: command_pool_alloc
        }
    }

    pub fn record_command(&self) {
        let command_buffer_begin_info = CommandBufferBeginInfo::default();
        self.pool_alloc.
    }
}
