use crate::device::PhysicalDevice;
use crate::GraphicContext;
use crate::VulkanObject;

use ash::{version::DeviceV1_0, vk, Device};

pub struct CommandPool {
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: &Device, physical_device: &PhysicalDevice) -> Self {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(*physical_device.graphics_index())
            .build();

        let command_pool = unsafe { device.create_command_pool(&pool_info, None).unwrap() };

        CommandPool { command_pool: command_pool }
    }
}

impl VulkanObject for CommandPool {
    type Object = vk::CommandPool;

    fn vulkan_object(&self) -> &Self::Object {
        &self.command_pool
    }

    fn cleanup(&self, _context: &GraphicContext) {
        unsafe {
            _context.get_device().destroy_command_pool(self.command_pool, None);
        }
    }
}
