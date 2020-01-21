use super::CommandPool;
use crate::Renderer;
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
}

impl VulkanObject for CommandBuffer {
    type Object = Vec<vk::CommandBuffer>;

    fn vulkan_object(&self) -> &Self::Object {
        &self.command_buffers
    }

    fn cleanup(&self, _renderer: &Renderer) {
        unsafe {
            _renderer.get_device().free_command_buffers(*_renderer.get_command_pool(), &self.command_buffers);
        }
    }
}