use crate::utility;
use crate::DebugMessenger;
use crate::PhysicalDevice;
use crate::Renderer;
use crate::VulkanObject;

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Instance,
};

use std::collections::HashSet;

pub struct LogicalDevice {
    logical_device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl LogicalDevice {
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice) -> Self {
        let queue_families: HashSet<&u32> = vec![physical_device.graphics_index(), physical_device.present_index()].into_iter().collect();
        let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();

        let priority = [1f32];
        for &&queue_index in queue_families.iter() {
            queue_create_infos.push(vk::DeviceQueueCreateInfo::builder().queue_family_index(queue_index).queue_priorities(&priority).build());
        }

        let device_features = vk::PhysicalDeviceFeatures::default();
        let extensions = PhysicalDevice::required_extension_names();
        let (_names, validation_layers) = DebugMessenger::get_validation_layers_vk();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(if utility::validation_enabled() { &validation_layers } else { &[] })
            .build();

        let device: Device = unsafe { instance.create_device(*physical_device.vulkan_object(), &create_info, None).expect("Failed to create logical device") };
        let graphics_queue = unsafe { device.get_device_queue(*physical_device.graphics_index(), 0) };
        let present_queue = unsafe { device.get_device_queue(*physical_device.present_index(), 0) };

        LogicalDevice {
            logical_device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
        }
    }
}

impl VulkanObject for LogicalDevice {
    type Object = Device;

    fn vulkan_object(&self) -> &Self::Object {
        &self.logical_device
    }

    fn cleanup(&self, _renderer: &Renderer) {
        unsafe {
            self.logical_device.destroy_device(None);
        }
    }
}
