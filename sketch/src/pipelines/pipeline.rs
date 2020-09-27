use super::shader;
use crate::{models::Vertex, renderpasses::RenderPass, GraphicContext, VulkanObject};

use ash::{version::DeviceV1_0, vk, Device};

use std::ffi::CString;

pub struct Pipeline {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn new(device: &Device, render_pass: &RenderPass, descriptor_layout: vk::DescriptorSetLayout) -> Pipeline {
        let vert_shader = shader::create_shader_module("assets/gen/shaders/shader.vert.spv", device).unwrap();
        let frag_shader = shader::create_shader_module("assets/gen/shaders/shader.frag.spv", device).unwrap();

        let entry_point_name = CString::new("main").unwrap();

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader)
            .name(&entry_point_name)
            .build();

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader)
            .name(&entry_point_name)
            .build();

        let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

        let vertex_binding_descriptions = [Vertex::get_binding_description()];
        let vertex_attribute_descriptions = Vertex::get_attribute_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .build();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder().viewport_count(1).scissor_count(1).build();

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1f32)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .build();

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1f32)
            .build();

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false)
            .build();

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder().logic_op_enable(false).attachments(&[color_blend_attachment]).build();

        let set_layouts = [descriptor_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts).build();

        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None).unwrap() };

        let dynamic_states = vec![vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
        let dynamic_states_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states).build();

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(*render_pass.vulkan_object())
            .dynamic_state(&dynamic_states_info)
            .subpass(0)
            .build();

        let pipeline = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None).unwrap()[0] };

        unsafe {
            device.destroy_shader_module(vert_shader, None);
            device.destroy_shader_module(frag_shader, None);
        }

        Pipeline { pipeline_layout, pipeline }
    }

    pub fn get_layout(&self) -> &vk::PipelineLayout {
        &self.pipeline_layout
    }
}

impl VulkanObject for Pipeline {
    type Object = vk::Pipeline;

    fn vulkan_object(&self) -> &Self::Object {
        &self.pipeline
    }

    fn cleanup(&self, _context: &GraphicContext) {
        unsafe {
            _context.get_device().destroy_pipeline(self.pipeline, None);
            _context.get_device().destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
