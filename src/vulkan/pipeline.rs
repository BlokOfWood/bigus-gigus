use std::{ffi::CString, ptr::null};

use ash::vk::{
    Buffer, ColorComponentFlags, CompareOp, CullModeFlags, DescriptorBufferInfo,
    DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize,
    DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateInfo, DescriptorType, DynamicState, FrontFace,
    GraphicsPipelineCreateInfo, ImageLayout, ImageView, LogicOp, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, PushConstantRange, RenderPass,
    Sampler, ShaderStageFlags, WriteDescriptorSet, FALSE, TRUE,
};

use crate::vulkan::{
    push_constants::PushConstant, ubo::UniformBufferObject, vulkan::MAX_FRAMES_IN_FLIGHT,
};

use super::{buffers::VertexBuffer, device_and_queues::BigusDevice, shader::Shaders};

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
impl BigusDevice {
    pub fn create_descriptor_pool(&self) -> DescriptorPool {
        let pool_sizes = [
            DescriptorPoolSize {
                ty: DescriptorType::UNIFORM_BUFFER,
                descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
            },
            DescriptorPoolSize {
                ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
            },
        ];

        unsafe {
            self.dev
                .create_descriptor_pool(
                    &DescriptorPoolCreateInfo {
                        pool_size_count: pool_sizes.len() as u32,
                        p_pool_sizes: pool_sizes.as_ptr(),
                        max_sets: MAX_FRAMES_IN_FLIGHT as u32,
                        ..Default::default()
                    },
                    None,
                )
                .unwrap()
        }
    }
    pub(super) fn create_descriptor_sets(
        &self,
        descriptor_set_layouts: DescriptorSetLayout,
        descriptor_pool: DescriptorPool,
        uniform_buffers: &Vec<Buffer>,
        texture_image_view: ImageView,
        texture_sampler: Sampler,
    ) -> Vec<DescriptorSet> {
        let layouts = [descriptor_set_layouts; MAX_FRAMES_IN_FLIGHT];

        let descriptor_sets = unsafe {
            self.dev
                .allocate_descriptor_sets(&DescriptorSetAllocateInfo {
                    descriptor_pool,
                    descriptor_set_count: layouts.len() as u32,
                    p_set_layouts: layouts.as_ptr(),
                    ..Default::default()
                })
                .unwrap()
        };

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = DescriptorBufferInfo {
                buffer: uniform_buffers[i],
                offset: 0,
                range: size_of::<UniformBufferObject>() as u64,
            };

            let image_info = DescriptorImageInfo {
                image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image_view: texture_image_view,
                sampler: texture_sampler,
            };

            let descriptor_writes = [
                WriteDescriptorSet {
                    dst_set: descriptor_sets[i],
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_type: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    p_buffer_info: &buffer_info,
                    ..Default::default()
                },
                WriteDescriptorSet {
                    dst_set: descriptor_sets[i],
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    p_image_info: &image_info,
                    ..Default::default()
                },
            ];

            unsafe { self.dev.update_descriptor_sets(&descriptor_writes, &[]) };
        }

        descriptor_sets
    }
}

pub(super) fn create_graphics_pipeline(
    device: &BigusDevice,
    render_pass: RenderPass,
    descriptor_set_layouts: Vec<DescriptorSetLayout>,
) -> (Pipeline, PipelineLayout) {
    let shaders = Shaders::new(&device, "src/shaders/vert.spv", "src/shaders/frag.spv");

    let vert_shader_c_name = CString::new("main").unwrap();
    let frag_shader_c_name = CString::new("main").unwrap();

    let vert_shader_stage_info = PipelineShaderStageCreateInfo {
        stage: ShaderStageFlags::VERTEX,
        module: shaders.vert_shader,
        p_name: vert_shader_c_name.as_ptr(),
        ..Default::default()
    };

    let frag_shader_stage_info = PipelineShaderStageCreateInfo {
        stage: ShaderStageFlags::FRAGMENT,
        module: shaders.frag_shader,
        p_name: frag_shader_c_name.as_ptr(),
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
        sample_shading_enable: TRUE,
        min_sample_shading: 0.2,
        rasterization_samples: device.max_sample_count,
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

    let dynamic_state = PipelineDynamicStateCreateInfo {
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr(),
        ..Default::default()
    };

    let push_constants = [PushConstantRange {
        stage_flags: ShaderStageFlags::VERTEX,
        offset: 0,
        size: size_of::<PushConstant>() as u32,
        ..Default::default()
    }];

    let pipeline_layout_info = PipelineLayoutCreateInfo {
        set_layout_count: 1,
        p_set_layouts: descriptor_set_layouts.as_ptr(),
        p_push_constant_ranges: push_constants.as_ptr(),
        push_constant_range_count: push_constants.len() as u32,
        ..Default::default()
    };

    let pipeline_layout = unsafe {
        device
            .dev
            .create_pipeline_layout(&pipeline_layout_info, None)
            .unwrap()
    };

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

    let graphics_pipeline = unsafe {
        device
            .dev
            .create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None)
            .unwrap()[0]
    };

    unsafe { device.dev.destroy_shader_module(shaders.frag_shader, None) };
    unsafe { device.dev.destroy_shader_module(shaders.vert_shader, None) };

    (graphics_pipeline, pipeline_layout)
}
