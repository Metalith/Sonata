use std::sync::Arc;

use crate::render::{device::Device, VulkanObject};

use ash::{version::DeviceV1_0, vk};

pub struct CommandPool {
    device: Arc<Device>,
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: Arc<Device>) -> Arc<Self> {
        trace!("Creating Command Pool");
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.physical_device().graphics_index())
            .build();

        let command_pool = unsafe { device.vk().create_command_pool(&pool_info, None).unwrap() };

        CommandPool { device, command_pool }.into()
    }
}

impl VulkanObject for CommandPool {
    type Object = vk::CommandPool;

    fn vk(&self) -> &Self::Object {
        &self.command_pool
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        trace!("Dropping Command Pool");
        unsafe {
            self.device.vk().destroy_command_pool(self.command_pool, None);
        }
    }
}
