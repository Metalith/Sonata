use crate::queue_family::QueueFamily;
use crate::VulkanObject; 

use ash::{
    vk,
    Instance,
    version::InstanceV1_0
};

pub struct PhysicalDevice {
    physical_device: vk::PhysicalDevice,
    queue_families: Vec<QueueFamily>,   
    graphics_index: u32
}

impl PhysicalDevice {
    pub fn new(instance: &Instance) -> Self {
        let physical_device = Self::pick_suitable_device(instance);
        let queue_families = QueueFamily::all(instance, physical_device);
        let graphics_index = Self::get_queue_indices(instance, physical_device).unwrap();


        PhysicalDevice {
            physical_device: physical_device,
            queue_families: queue_families,
            graphics_index: graphics_index
        }
    }

    fn pick_suitable_device(instance: &Instance) -> vk::PhysicalDevice {
        let physical_devices = unsafe { instance.enumerate_physical_devices().expect("Failed to enumerate physical devices") };
        debug!("{} devices (GPU) found with vulkan support.", physical_devices.len());

        let mut result = None;

        for &physical_device in physical_devices.iter() {
            if result.is_none() && Self::is_device_suitable(instance, physical_device) {
                result = Some(physical_device)
            }
        }

        match result {
            None => panic!("No suitable physical device"),
            Some(device) => device
        }
    }

    fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        Self::get_queue_indices(instance, device).is_ok()
    }

    fn get_queue_indices(instance: &Instance, device: vk::PhysicalDevice) -> Result<u32, &'static str> {
        let queue_families = QueueFamily::all(instance, device);

        let mut graphics_index = None;
        for queue_family in queue_families.iter() {
            if graphics_index.is_none() && queue_family.count > 0 && queue_family.flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_index = Some(queue_family.index);
            }
        }

        match graphics_index {
            Some(g_index) => Ok(g_index),
            None => Err("Graphics queue not present")
        }
    }

    pub fn graphics_index(&self) -> &u32 {
        &self.graphics_index
    }

    pub fn queue_families(&self) -> &Vec<QueueFamily> {
        &self.queue_families
    }
}

impl VulkanObject for PhysicalDevice {
    type Object = vk::PhysicalDevice;

    fn vulkan_object(&self) -> &Self::Object {
        &self.physical_device
    }

    fn cleanup(&self) {}
}