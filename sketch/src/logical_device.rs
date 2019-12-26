use crate::VulkanObject;
use crate::PhysicalDevice;
use crate::DebugMessenger;

use ash::{
    vk,
    Instance,
    Device,
    extensions::khr::Swapchain,
    version::{InstanceV1_0, DeviceV1_0}
};

pub struct LogicalDevice {
    logical_device: Device,
}

impl LogicalDevice {
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice) -> Self {
        let validation_enabled: bool  = if std::env::var("WIND_VK_VALIDATION").is_ok() { std::env::var("WIND_VK_VALIDATION").unwrap().parse().unwrap() } else { false };

        let queue_families = [physical_device.graphics_index()];
        let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();

        let priority = [1f32];
        for &&queue_index in queue_families.iter() {
            queue_create_infos.push(vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_index)
                .queue_priorities(&priority)
                .build());
        }

        let device_features = vk::PhysicalDeviceFeatures::default();
        let extensions = Self::required_extension_names();
        let (_names, validation_layers)= DebugMessenger::get_validation_layers_vk();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(if validation_enabled { &validation_layers } else { &[] })
            .build();

        let device = unsafe { instance.create_device(*physical_device.vulkan_object(), &create_info, None).expect("Failed to create logical device") };

        LogicalDevice {
            logical_device: device
        }
    }

    fn required_extension_names() -> Vec<*const i8> {
        vec![
            Swapchain::name().as_ptr()
        ]
    }
}

impl VulkanObject for LogicalDevice {
    type Object = Device;

    fn vulkan_object(&self) -> &Self::Object {
        &self.logical_device
    }

    fn cleanup(&self) {
        unsafe {
            self.logical_device.destroy_device(None);
        }
    }
}