use std::{ptr::null, sync::Arc};

use ash::vk::{
    ColorComponentFlags, CompareOp, CullModeFlags, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, DynamicState, FrontFace, GraphicsPipelineCreateInfo, LogicOp, Pipeline, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, RenderPass, SampleCountFlags, ShaderStageFlags, FALSE, TRUE
};

use super::{
    buffers::{Vertex, VertexBuffer},
    device_and_queues::BigusDevice,
    shader::Shaders,
};

pub(super) fn create_descriptor_set_layout(device: &BigusDevice) -> DescriptorSetLayout {
    let ubo_layout_binding = DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: 1,
        descriptor_type: DescriptorType::UNIFORM_BUFFER,
        p_immutable_samplers: null(),
        stage_flags: ShaderStageFlags::VERTEX,
        ..Default::default()
    };

    let sampler_layout_binding = DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_count: 1,
        descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
        p_immutable_samplers: null(),
        stage_flags: ShaderStageFlags::FRAGMENT,
        ..Default::default()
    };

    let bindings = [ubo_layout_binding, sampler_layout_binding];

    let layout_create_info = DescriptorSetLayoutCreateInfo {
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };

    return unsafe {
        device
            .dev
            .create_descriptor_set_layout(&layout_create_info, None)
            .unwrap()
    };
}
/*
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
*/
pub(super) fn create_graphics_pipeline(
    device: &BigusDevice,
    render_pass: RenderPass,
    descriptor_set_layouts: Vec<DescriptorSetLayout>,
) -> Pipeline {
    let shaders = Shaders::new(&device, "src/shaders/vert.spv", "src/shaders/frag.spv");

    let vert_shader_stage_info = PipelineShaderStageCreateInfo {
        stage: ShaderStageFlags::VERTEX,
        module: shaders.vert_shader,
        p_name: "main".as_ptr() as *const i8,
        ..Default::default()
    };

    let frag_shader_stage_info = PipelineShaderStageCreateInfo {
        stage: ShaderStageFlags::FRAGMENT,
        module: shaders.frag_shader,
        p_name: "main".as_ptr() as *const i8,
        ..Default::default()
    };

    let binding_description = VertexBuffer::get_binding_description();
    let attribute_description =
        VertexBuffer::get_attribute_descriptions().map(|description| description.1);

    let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

    let vertex_input_info = PipelineVertexInputStateCreateInfo {
        vertex_binding_description_count: 1,
        vertex_attribute_description_count: attribute_description.len() as u32,
        p_vertex_binding_descriptions: &binding_description,
        p_vertex_attribute_descriptions: attribute_description.as_ptr(),
        ..Default::default()
    };

    let input_assembly = PipelineInputAssemblyStateCreateInfo {
        topology: PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: FALSE,
        ..Default::default()
    };

    let viewport_state = PipelineViewportStateCreateInfo {
        viewport_count: 1,
        scissor_count: 1,
        ..Default::default()
    };

    let rasterizer = PipelineRasterizationStateCreateInfo {
        depth_clamp_enable: FALSE,
        rasterizer_discard_enable: FALSE,
        polygon_mode: PolygonMode::FILL,
        line_width: 1.0,
        cull_mode: CullModeFlags::BACK,
        front_face: FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: FALSE,
        ..Default::default()
    };

    let multisampling = PipelineMultisampleStateCreateInfo {
        sample_shading_enable: FALSE,
        rasterization_samples: SampleCountFlags::TYPE_1,
        ..Default::default()
    };

    let depth_stencil = PipelineDepthStencilStateCreateInfo {
        depth_test_enable: TRUE,
        depth_write_enable: TRUE,
        depth_compare_op: CompareOp::LESS,
        depth_bounds_test_enable: FALSE,
        stencil_test_enable: FALSE,
        ..Default::default()
    };

    let color_blend_attachment = PipelineColorBlendAttachmentState {
        color_write_mask: ColorComponentFlags::RGBA,
        blend_enable: FALSE,
        ..Default::default()
    };

    let color_blending = PipelineColorBlendStateCreateInfo {
        logic_op_enable: FALSE,
        logic_op: LogicOp::COPY,
        attachment_count: 1,
        p_attachments: &color_blend_attachment,
        blend_constants: [0.0, 0.0, 0.0, 0.0],
        ..Default::default()
    };

    let dynamic_states = [DynamicState::VIEWPORT, DynamicState::SCISSOR];

    let dynamic_state = PipelineDynamicStateCreateInfo{
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr(),
        ..Default::default()
    };

    let pipeline_layout_info = PipelineLayoutCreateInfo {
        set_layout_count: 1,
        p_set_layouts: descriptor_set_layouts.as_ptr(),
        ..Default::default()
    };

    let pipeline_layout = unsafe { device.dev.create_pipeline_layout(&pipeline_layout_info, None).unwrap() };

    let pipeline_info = GraphicsPipelineCreateInfo {
        stage_count: 2,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_info,
        p_input_assembly_state: &input_assembly,
        p_viewport_state: &viewport_state,
        p_rasterization_state: &rasterizer,
        p_multisample_state: &multisampling,
        p_color_blend_state: &color_blending,
        p_depth_stencil_state: &depth_stencil,
        p_dynamic_state: &dynamic_state,
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        base_pipeline_handle: Pipeline::null(),
        ..Default::default()
    };

    let graphics_pipeline = unsafe { device.dev.create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None).unwrap()[0] };

    unsafe { device.dev.destroy_shader_module(shaders.frag_shader, None) };
    unsafe { device.dev.destroy_shader_module(shaders.vert_shader, None) };

    graphics_pipeline

}
