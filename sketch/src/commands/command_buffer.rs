use super::CommandPool;
use crate::{GraphicContext, VulkanObject};

use ash::{version::DeviceV1_0, vk, Device};

pub struct CommandBuffer {
    command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandBuffer {
    pub fn new(device: &Device, command_pool: &CommandPool, count: u32) -> Self {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*command_pool.vulkan_object())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count)
            .build();

        let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info).unwrap() };

        CommandBuffer { command_buffers }
    }

    pub fn begin(&self, index: usize, device: &Device, render_pass_info: &vk::RenderPassBeginInfo) {
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            device.begin_command_buffer(self.command_buffers[index], &begin_info).unwrap();
            device.cmd_begin_render_pass(self.command_buffers[index], &render_pass_info, vk::SubpassContents::INLINE);
        };
    }

    pub fn bind_pipeline(&self, index: usize, device: &Device, pipeline: &vk::Pipeline) {
        unsafe {
            device.cmd_bind_pipeline(self.command_buffers[index], vk::PipelineBindPoint::GRAPHICS, *pipeline);
        }
    }

    pub fn set_viewport(&self, index: usize, device: &Device, viewport: vk::Viewport) {
        unsafe {
            device.cmd_set_viewport(self.command_buffers[index], 0, &[viewport]);
        }
    }

    pub fn set_scissor(&self, index: usize, device: &Device, scissor: vk::Rect2D) {
        unsafe {
            device.cmd_set_scissor(self.command_buffers[index], 0, &[scissor]);
        }
    }

    pub fn bind_descriptor_sets(&self, index: usize, device: &Device, pipeline_layout: &vk::PipelineLayout, descriptor_sets: &[vk::DescriptorSet]) {
        unsafe {
            let null = [];
            device.cmd_bind_descriptor_sets(self.command_buffers[index], vk::PipelineBindPoint::GRAPHICS, *pipeline_layout, 0, descriptor_sets, &null);
        }
    }

    pub fn end(&self, index: usize, device: &Device) {
        unsafe {
            device.cmd_end_render_pass(self.command_buffers[index]);
            device.end_command_buffer(self.command_buffers[index]).unwrap();
        };
    }

    pub fn get(&self, image_index: usize) -> &vk::CommandBuffer {
        &self.command_buffers[image_index]
    }
}

impl VulkanObject for CommandBuffer {
    type Object = Vec<vk::CommandBuffer>;

    fn vulkan_object(&self) -> &Self::Object {
        &self.command_buffers
    }

    fn cleanup(&self, _context: &GraphicContext) {
        unsafe {
            _context.get_device().free_command_buffers(*_context.get_command_pool(), &self.command_buffers);
        }
    }
}
