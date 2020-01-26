use super::CommandPool;
use crate::GraphicContext;
use crate::VulkanObject;

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

        CommandBuffer { command_buffers: command_buffers }
    }

    pub fn begin(&self, index: usize, device: &Device, render_pass_info: &vk::RenderPassBeginInfo, viewport: vk::Viewport, scissor: vk::Rect2D, pipeline: &vk::Pipeline) {
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            device.begin_command_buffer(self.command_buffers[index], &begin_info).unwrap();
            device.cmd_begin_render_pass(self.command_buffers[index], &render_pass_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(self.command_buffers[index], vk::PipelineBindPoint::GRAPHICS, *pipeline); //TODO: Remove this
            device.cmd_set_viewport(self.command_buffers[index], 0, &[viewport]);
            device.cmd_set_scissor(self.command_buffers[index], 0, &[scissor]);
        };
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
