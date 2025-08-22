use ash::vk::{
    AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
    CommandBuffer, CommandBufferAllocateInfo,
    CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags, CommandPool,
    CommandPoolCreateFlags, CommandPoolCreateInfo, Fence, Format, ImageLayout, PipelineBindPoint, PipelineStageFlags, RenderPass,
    RenderPassCreateInfo, SampleCountFlags, SubmitInfo, SubpassDependency, SubpassDescription, SUBPASS_EXTERNAL,
};

use crate::vulkan::vulkan::MAX_FRAMES_IN_FLIGHT;

use super::device_and_queues::{BigusDevice, QueueFamilyIndices};

impl BigusDevice {
    pub(super) fn create_command_pool(
        &self,
        queue_family_indices: &QueueFamilyIndices,
    ) -> CommandPool {
        let pool_info = CommandPoolCreateInfo {
            flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_family_indices.graphics_family.unwrap() as u32,
            ..Default::default()
        };

        unsafe { self.dev.create_command_pool(&pool_info, None).unwrap() }
    }

    pub(super) fn begin_single_time_commands(&self, command_pool: &CommandPool) -> CommandBuffer {
        let command_buffer = unsafe {
            self.dev
                .allocate_command_buffers(&CommandBufferAllocateInfo {
                    level: CommandBufferLevel::PRIMARY,
                    command_pool: *command_pool,
                    command_buffer_count: 1,
                    ..Default::default()
                })
                .unwrap()[0]
        };

        unsafe {
            self.dev
                .begin_command_buffer(
                    command_buffer,
                    &CommandBufferBeginInfo {
                        flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                        ..Default::default()
                    },
                )
                .unwrap()
        };

        return command_buffer;
    }

    pub(super) fn end_single_time_commands(
        &self,
        command_pool: &CommandPool,
        command_buffer: CommandBuffer,
    ) {
        unsafe { self.dev.end_command_buffer(command_buffer).unwrap() }

        unsafe {
            self.dev
                .queue_submit(
                    self.queues.graphics_queue,
                    &[SubmitInfo {
                        command_buffer_count: 1,
                        p_command_buffers: &command_buffer,
                        ..Default::default()
                    }],
                    Fence::null(),
                )
                .unwrap();

            self.dev
                .queue_wait_idle(self.queues.graphics_queue)
                .unwrap();

            self.dev
                .free_command_buffers(*command_pool, &[command_buffer]);
        };
    }

    pub(super) fn create_command_buffers(&self, command_pool: CommandPool) -> Vec<CommandBuffer> {
        unsafe {
            self.dev
                .allocate_command_buffers(&CommandBufferAllocateInfo {
                    command_pool,
                    level: CommandBufferLevel::PRIMARY,
                    command_buffer_count: MAX_FRAMES_IN_FLIGHT as u32,
                    ..Default::default()
                })
                .unwrap()
        }
    }

}
/*
impl CommandPool {
    pub fn new(dev: BigusDevice, surface: Arc<Surface>) -> Self {
        let queue_families =
            QueueFamilyIndices::find_queue_families(dev.phys_device().clone(), surface);

        let command_buffer_allocator_info = StandardCommandBufferAllocatorCreateInfo {
            primary_buffer_count: 2,
            secondary_buffer_count: 0,
            ..Default::default()
        };

        CommandPool {
            command_buffer_allocator: StandardCommandBufferAllocator::new(
                dev.device(),
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
        descriptor_set: Arc<PersistentDescriptorSet>,
        extent: [u32; 2],
        vertex_buffer: Arc<Buffer>,
        index_buffer: Arc<Buffer>,
    ) -> Arc<PrimaryAutoCommandBuffer> {
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
        render_pass_info.clear_values = vec![
            Some(ClearValue::Float([0.0f32, 0.0f32, 0.0f32, 1.0f32])),
            Some(ClearValue::Depth(1.0)),
        ];

        command_builder
            .begin_render_pass(
                render_pass_info,
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap();

        command_builder
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap();
        command_builder
            .bind_vertex_buffers(0, Subbuffer::new(vertex_buffer.clone()))
            .unwrap();

        if let Err(err) = command_builder
            .bind_index_buffer(IndexBuffer::U16(Subbuffer::new(index_buffer).reinterpret()))
        {
            println!("{}", err);
        }

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

        command_builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                vec![descriptor_set],
            )
            .unwrap();

        command_builder
            .draw_indexed(INDICES.len() as u32, 1, 0, 0, 0)
            .unwrap();

        command_builder
            .end_render_pass(SubpassEndInfo::default())
            .unwrap();

        command_builder.build().unwrap()
    }

    pub fn record_copy_pass(
        &self,
        src_buffer: Subbuffer<[u8]>,
        dst_buffer: Subbuffer<[u8]>,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let mut transfer_command_buffer = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graph_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        transfer_command_buffer
            .copy_buffer(CopyBufferInfo::buffers(src_buffer, dst_buffer))
            .unwrap();

        transfer_command_buffer.build().unwrap()
    }
}
*/
pub(super) fn create_render_pass(
    device: &BigusDevice,
    image_format: Format,
    depth_format: Format,
) -> RenderPass {
    let color_attachment = AttachmentDescription {
        format: image_format,
        samples: SampleCountFlags::TYPE_1,
        load_op: AttachmentLoadOp::CLEAR,
        store_op: AttachmentStoreOp::STORE,
        stencil_load_op: AttachmentLoadOp::DONT_CARE,
        stencil_store_op: AttachmentStoreOp::DONT_CARE,
        initial_layout: ImageLayout::UNDEFINED,
        final_layout: ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    let depth_attchment = AttachmentDescription {
        format: depth_format,
        samples: SampleCountFlags::TYPE_1,
        load_op: AttachmentLoadOp::CLEAR,
        store_op: AttachmentStoreOp::DONT_CARE,
        stencil_load_op: AttachmentLoadOp::DONT_CARE,
        stencil_store_op: AttachmentStoreOp::DONT_CARE,
        initial_layout: ImageLayout::UNDEFINED,
        final_layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let color_attachment_ref = AttachmentReference {
        attachment: 0,
        layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let depth_attachment_ref = AttachmentReference {
        attachment: 1,
        layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let subpass = SubpassDescription {
        pipeline_bind_point: PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_depth_stencil_attachment: &depth_attachment_ref,
        ..Default::default()
    };

    let subpass_dependency = SubpassDependency {
        src_subpass: SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        src_access_mask: AccessFlags::empty(),
        dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        dst_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE
            | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        ..Default::default()
    };

    let attachments = vec![color_attachment, depth_attchment];

    let render_pass_create_info = RenderPassCreateInfo {
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &subpass_dependency,
        ..Default::default()
    };

    unsafe {
        device
            .dev
            .create_render_pass(&render_pass_create_info, None)
            .unwrap()
    }
}
