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
            | PipelineStageFlags::LATE_FRAGMENT_TESTS,
        src_access_mask: AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
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
