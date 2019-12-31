use crate::VulkanObject;
use crate::Renderer;
use crate::CommandPool;

use ash::{
    vk,
    Device,
    version::DeviceV1_0
};

pub struct CommandBuffer {
    command_buffers: Vec<vk::CommandBuffer>
}

impl CommandBuffer {
    pub fn new(device: &Device, command_pool: &CommandPool, count: u32) -> Self {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*command_pool.vulkan_object())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count)
            .build();
        
        let command_buffers = unsafe {
            device.allocate_command_buffers(&alloc_info).unwrap()
        };

        CommandBuffer {
            command_buffers: command_buffers
        }
    }
}

impl VulkanObject for CommandBuffer {
    type Object = Vec<vk::CommandBuffer>;

    fn vulkan_object(&self) -> &Self::Object {
        &self.command_buffers
    }

    fn cleanup(&self, _renderer: &Renderer) {
    }
}