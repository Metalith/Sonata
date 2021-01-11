use std::sync::Arc;

use crate::render::{device::Device, VulkanObject};

use ash::{version::DeviceV1_0, vk};

pub struct DescriptorLayout {
    device: Arc<Device>,
    descriptor_layout: vk::DescriptorSetLayout,
}

impl DescriptorLayout {
    pub fn new(device: Arc<Device>) -> Arc<DescriptorLayout> {
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let ubo_layout_bindings = [ubo_layout_binding];

        let descriptor_layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_layout_bindings).build();

        let descriptor_layout = unsafe { device.vk().create_descriptor_set_layout(&descriptor_layout_info, None).unwrap() };

        DescriptorLayout { device, descriptor_layout }.into()
    }
}

impl VulkanObject for DescriptorLayout {
    type Object = vk::DescriptorSetLayout;

    fn vk(&self) -> &Self::Object {
        &self.descriptor_layout
    }
}

impl Drop for DescriptorLayout {
    fn drop(&mut self) {
        trace!("Dropping Descriptor Layout");
        unsafe {
            self.device.vk().destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
    }
}
