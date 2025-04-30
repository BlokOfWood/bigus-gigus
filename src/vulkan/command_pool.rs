use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, IndexBuffer, Subbuffer}, command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
    }, descriptor_set::PersistentDescriptorSet, device::{physical::PhysicalDevice, Device}, format::{ClearValue, Format}, image::{ImageLayout, SampleCount}, pipeline::{
        graphics::viewport::{Scissor, Viewport},
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    }, render_pass::{AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Framebuffer, RenderPass, RenderPassCreateInfo, SubpassDependency, SubpassDescription}, swapchain::Surface, sync::{AccessFlags, PipelineStages}
};

use super::{buffers::INDICES, device_and_queues::QueueFamilyIndices};

pub struct CommandPool {
    command_buffer_allocator: StandardCommandBufferAllocator,
    graph_family_index: u32,
}

impl CommandPool {
    pub fn new(dev: Arc<Device>, phys_dev: Arc<PhysicalDevice>, surface: Arc<Surface>) -> Self {
        let queue_families = QueueFamilyIndices::find_queue_families(phys_dev, surface);

        let command_buffer_allocator_info = StandardCommandBufferAllocatorCreateInfo {
            primary_buffer_count: 2,
            secondary_buffer_count: 0,
            ..Default::default()
        };

        CommandPool {
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

pub(super) fn create_render_pass(dev: Arc<Device>, image_format: Format) -> Arc<RenderPass> {
    let color_attachment = AttachmentDescription {
        format: image_format,
        samples: SampleCount::Sample1,
        load_op: AttachmentLoadOp::Clear,
        store_op: AttachmentStoreOp::Store,
        stencil_load_op: Some(AttachmentLoadOp::DontCare),
        stencil_store_op: Some(AttachmentStoreOp::DontCare),
        initial_layout: ImageLayout::Undefined,
        final_layout: ImageLayout::PresentSrc,
        ..Default::default()
    };

    let subpass_dependency = SubpassDependency {
        src_subpass: None,
        dst_subpass: Some(0),
        src_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
        src_access: AccessFlags::empty(),
        dst_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
        dst_access: AccessFlags::COLOR_ATTACHMENT_WRITE,
        ..Default::default()
    };

    let color_attachment_ref = AttachmentReference {
        attachment: 0,
        layout: ImageLayout::ColorAttachmentOptimal,
        ..Default::default()
    };

    let subpass_desc = SubpassDescription {
        color_attachments: vec![Some(color_attachment_ref)],
        ..Default::default()
    };

    let render_pass_create_info = RenderPassCreateInfo {
        attachments: vec![color_attachment],
        subpasses: vec![subpass_desc],
        dependencies: vec![subpass_dependency],
        ..Default::default()
    };

    RenderPass::new(dev, render_pass_create_info).unwrap()
}
