use std::{collections::BTreeMap, sync::Arc};

use vulkano::{
    buffer::Subbuffer, descriptor_set::{allocator::StandardDescriptorSetAllocator, layout::{
        DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
        DescriptorType,
    }, PersistentDescriptorSet, WriteDescriptorSet}, device::Device, image::{sampler::Sampler, view::ImageView, SampleCount}, pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState, ColorComponents},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
            subpass::PipelineSubpassType,
            vertex_input::VertexInputState,
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineCreateFlags, PipelineLayout,
        PipelineShaderStageCreateInfo,
    }, render_pass::{RenderPass, Subpass}, shader::{EntryPoint, ShaderStages}
};

use super::{shader::Shaders, ubo::UniformBufferObject, vertex_buffer::VertexBuffer};

pub(super) fn create_descriptor_set_layout(device: Arc<Device>) -> Arc<DescriptorSetLayout> {
    let ubo_layout_binding = DescriptorSetLayoutBinding {
        stages: ShaderStages::VERTEX,
        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
    };

    let sampler_layout_binding = DescriptorSetLayoutBinding {
        stages: ShaderStages::FRAGMENT,
        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::CombinedImageSampler)
    };

    let layout_create_info = DescriptorSetLayoutCreateInfo {
        bindings: BTreeMap::from([(0, ubo_layout_binding), (1, sampler_layout_binding)]),
        ..Default::default()
    };

    return DescriptorSetLayout::new(device, layout_create_info).unwrap();
}

pub(super) fn create_descriptor_sets(
    layout: Arc<DescriptorSetLayout>,
    uniform_buffers: &[Subbuffer<UniformBufferObject>], // Pass in the buffersr_sets(
    device: Arc<Device>,
    image_view: Arc<ImageView>,
    sampler: Arc<Sampler>,
) -> Vec<Arc<PersistentDescriptorSet>> {
    let descriptor_set_allocator =
        StandardDescriptorSetAllocator::new(device.clone(), Default::default());
    let num_sets = uniform_buffers.len();
    let mut descriptor_sets: Vec<Arc<PersistentDescriptorSet>> = Vec::new();

    for i in 0..num_sets {
        let set = PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, uniform_buffers[i].clone()),
                WriteDescriptorSet::image_view_sampler(1, image_view.clone(), sampler.clone()),
            ],
            [],
        )
        .unwrap(); // Handle potential errors
        descriptor_sets.push(set);
    }

    descriptor_sets
}

pub(super) fn create_graphics_pipeline(
    dev: Arc<Device>,
    render_pass: Arc<RenderPass>,
    image_extent: [u32; 2],
    descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
) -> (Arc<PipelineLayout>, Arc<GraphicsPipeline>) {
    let shaders = Shaders::new(dev.clone(), "src/shaders/vert.spv", "src/shaders/frag.spv");

    let vert_entry_point: EntryPoint = shaders.vert_shader.single_entry_point().unwrap();
    let vert_stage_info = PipelineShaderStageCreateInfo::new(vert_entry_point);
    let frag_entry_point: EntryPoint = shaders.frag_shader.single_entry_point().unwrap();
    let frag_stage_info = PipelineShaderStageCreateInfo::new(frag_entry_point);

    let vertex_input_state = VertexInputState::new()
        .binding(0, VertexBuffer::get_binding_description())
        .attributes(VertexBuffer::get_attribute_descriptions());

    let input_assembly_state = InputAssemblyState {
        topology: PrimitiveTopology::TriangleList,
        primitive_restart_enable: false,
        ..Default::default()
    };
    let viewport = Viewport {
        offset: [0f32, 0f32],
        extent: [image_extent[0] as f32, image_extent[1] as f32],
        depth_range: 0f32..=1f32,
    };
    let scissor = Scissor {
        offset: [0, 0],
        extent: image_extent,
    };
    let viewport_state = ViewportState {
        viewports: [viewport].into(),
        scissors: [scissor].into(),
        ..Default::default()
    };

    let rasterization_state = RasterizationState {
        depth_clamp_enable: false,
        rasterizer_discard_enable: false,
        polygon_mode: PolygonMode::Fill,
        line_width: 1.0f32,
        cull_mode: CullMode::Back,
        front_face: FrontFace::CounterClockwise,
        depth_bias: None,
        ..Default::default()
    };

    let multisample_state = MultisampleState {
        sample_shading: None,
        rasterization_samples: SampleCount::Sample1,
        ..Default::default()
    };

    let color_blend_attachment = ColorBlendAttachmentState {
        color_write_enable: true,
        color_write_mask: ColorComponents::all(),
        blend: None,
        ..Default::default()
    };

    let color_blending = ColorBlendState {
        logic_op: None,
        attachments: vec![color_blend_attachment],
        ..Default::default()
    };

    let pipeline_layout_info = PipelineLayoutCreateInfo {
        set_layouts: descriptor_set_layouts,
        ..Default::default()
    };

    let pipeline_layout = PipelineLayout::new(dev.clone(), pipeline_layout_info).unwrap();

    let subpass = Subpass::from(render_pass, 0).unwrap();

    let mut graphics_pipeline_create_info =
        GraphicsPipelineCreateInfo::layout(pipeline_layout.clone());
    graphics_pipeline_create_info.stages = vec![vert_stage_info, frag_stage_info].into();
    graphics_pipeline_create_info.vertex_input_state = Some(vertex_input_state);
    graphics_pipeline_create_info.input_assembly_state = Some(input_assembly_state);
    graphics_pipeline_create_info.viewport_state = Some(viewport_state);
    graphics_pipeline_create_info.rasterization_state = Some(rasterization_state);
    graphics_pipeline_create_info.multisample_state = Some(multisample_state);
    graphics_pipeline_create_info.depth_stencil_state = None;
    graphics_pipeline_create_info.color_blend_state = Some(color_blending);
    graphics_pipeline_create_info.dynamic_state =
        ahash::HashSet::from_iter([DynamicState::Viewport, DynamicState::Scissor]);
    graphics_pipeline_create_info.subpass = Some(PipelineSubpassType::BeginRenderPass(subpass));
    graphics_pipeline_create_info.base_pipeline = None;
    graphics_pipeline_create_info.tessellation_state = None;
    graphics_pipeline_create_info.discard_rectangle_state = None;
    graphics_pipeline_create_info.flags = PipelineCreateFlags::empty();

    (
        pipeline_layout,
        GraphicsPipeline::new(dev.clone(), None, graphics_pipeline_create_info).unwrap(),
    )
}
