use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        pool::{
            CommandBufferAllocateInfo, CommandPoolAlloc, CommandPoolCreateFlags,
            CommandPoolCreateInfo, CommandPoolResetFlags,
        },
        AutoCommandBufferBuilder, CommandBufferLevel, CommandBufferUsage, PrimaryAutoCommandBuffer,
        PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
    },
    device::{physical::PhysicalDevice, Device},
    format::ClearValue,
    pipeline::{
        self,
        graphics::viewport::{Scissor, Viewport},
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, RenderPass},
    swapchain::Surface,
};

use super::queue_family::QueueFamilyIndices;

pub struct CommandPool {
    command_pool: vulkano::command_buffer::pool::CommandPool,
    command_buffer_allocator: StandardCommandBufferAllocator,
    graph_family_index: u32,
}

impl CommandPool {
    pub fn new(dev: Arc<Device>, phys_dev: Arc<PhysicalDevice>, surface: Arc<Surface>) -> Self {
        let queue_families = QueueFamilyIndices::find_queue_families(phys_dev, surface);

        let command_pool_create_info = CommandPoolCreateInfo {
            queue_family_index: queue_families.graphics_family.unwrap(),
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };

        let command_buffer_allocator_info = StandardCommandBufferAllocatorCreateInfo {
            primary_buffer_count: 1,
            secondary_buffer_count: 0,
            ..Default::default()
        };

        CommandPool {
            command_pool: vulkano::command_buffer::pool::CommandPool::new(
                dev.clone(),
                command_pool_create_info,
            )
            .unwrap(),
            command_buffer_allocator: StandardCommandBufferAllocator::new(
                dev.clone(),
                command_buffer_allocator_info,
            ),
            graph_family_index: queue_families.graphics_family.unwrap(),
        }
    }

    pub fn record_render_pass(
        &self,
        render_pass: Arc<RenderPass>,
        framebuffer: Arc<Framebuffer>,
        pipeline: Arc<GraphicsPipeline>,
        extent: [u32; 2],
    ) -> Arc<PrimaryAutoCommandBuffer> {
        unsafe {
            self.command_pool
                .reset(CommandPoolResetFlags::empty())
                .unwrap()
        };

        let mut command_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graph_family_index,
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();

        let mut render_pass_info = RenderPassBeginInfo::framebuffer(framebuffer);
        render_pass_info.render_pass = render_pass;
        render_pass_info.render_area_offset = [0, 0];
        render_pass_info.render_area_extent = extent;
        render_pass_info.clear_values =
            vec![Some(ClearValue::Float([0.0f32, 0.0f32, 0.0f32, 1.0f32]))];

        command_builder
            .begin_render_pass(
                render_pass_info,
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap();

        command_builder.bind_pipeline_graphics(pipeline).unwrap();

        let viewport: Viewport = Viewport {
            offset: [0.0f32, 0.0f32],
            depth_range: 0.0f32..=1.0f32,
            extent: [extent[0] as f32, extent[1] as f32],
        };
        command_builder
            .set_viewport(0, vec![viewport].into())
            .unwrap();

        let scissor = Scissor {
            extent,
            offset: [0, 0],
        };
        command_builder
            .set_scissor(0, vec![scissor].into())
            .unwrap();

        command_builder.draw(3, 1, 0, 0).unwrap();

        command_builder.end_render_pass(SubpassEndInfo::default()).unwrap();

        command_builder.build().unwrap()
    }
}
