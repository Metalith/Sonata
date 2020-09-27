use super::DescriptorPool;
use crate::{
    buffers::{UniformBufferObject, UniformTestObject},
    GraphicContext, VulkanObject,
};

use ash::{version::DeviceV1_0, vk, Device};

pub struct DescriptorSet {
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl DescriptorSet {
    pub fn new(device: &Device, descriptor_layout: vk::DescriptorSetLayout, image_count: u32, descriptor_pool: &DescriptorPool, ubos: &[UniformBufferObject]) -> DescriptorSet {
        let layouts = (0..image_count).map(|_| descriptor_layout).collect::<Vec<_>>();

        let alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(*descriptor_pool.vulkan_object()).set_layouts(&layouts).build();

        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap() };

        descriptor_sets.iter().zip(ubos.iter()).for_each(|(set, buffer)| {
            let buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(*buffer.vulkan_object())
                .offset(0)
                .range(UniformTestObject::get_size() as vk::DeviceSize)
                .build();

            let buffer_infos = [buffer_info];

            let descriptor_write = vk::WriteDescriptorSet::builder()
                .dst_set(*set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build();
            let descriptor_writes = [descriptor_write];

            let null = [];

            unsafe { device.update_descriptor_sets(&descriptor_writes, &null) }
        });

        DescriptorSet { descriptor_sets }
    }
}

impl VulkanObject for DescriptorSet {
    type Object = Vec<vk::DescriptorSet>;

    fn vulkan_object(&self) -> &Self::Object {
        &self.descriptor_sets
    }

    fn cleanup(&self, _context: &GraphicContext) {}
}
