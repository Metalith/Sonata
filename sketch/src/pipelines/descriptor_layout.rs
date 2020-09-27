use crate::{GraphicContext, VulkanObject};

use ash::{version::DeviceV1_0, vk, Device};

pub struct DescriptorLayout {
    descriptor_layout: vk::DescriptorSetLayout,
}

impl DescriptorLayout {
    pub fn new(device: &Device) -> DescriptorLayout {
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let ubo_layout_bindings = [ubo_layout_binding];

        let descriptor_layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_layout_bindings).build();

        let descriptor_layout = unsafe { device.create_descriptor_set_layout(&descriptor_layout_info, None).unwrap() };

        DescriptorLayout { descriptor_layout }
    }
}

impl VulkanObject for DescriptorLayout {
    type Object = vk::DescriptorSetLayout;

    fn vulkan_object(&self) -> &Self::Object {
        &self.descriptor_layout
    }

    fn cleanup(&self, _context: &GraphicContext) {
        unsafe { _context.get_device().destroy_descriptor_set_layout(self.descriptor_layout, None) };
    }
}
