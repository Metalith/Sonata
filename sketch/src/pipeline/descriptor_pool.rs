use crate::GraphicContext;
use crate::VulkanObject;

use ash::{version::DeviceV1_0, vk, Device};

pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn new(device: &Device, image_count: u32) -> DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize::builder().descriptor_count(image_count).build()];

        let pool_info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(image_count);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() };

        DescriptorPool { descriptor_pool }
    }
}

impl VulkanObject for DescriptorPool {
    type Object = vk::DescriptorPool;

    fn vulkan_object(&self) -> &Self::Object {
        &self.descriptor_pool
    }

    fn cleanup(&self, _context: &GraphicContext) {
        unsafe {
            _context.get_device().destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
