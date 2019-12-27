use crate::queue_family::QueueFamily;
use crate::VulkanObject; 
use crate::Surface;

use ash::{
    vk,
    Instance,
    version::InstanceV1_0
};

pub struct PhysicalDevice {
    physical_device: vk::PhysicalDevice,
    queue_families: Vec<QueueFamily>,   
    graphics_index: u32,
    present_index: u32
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, surface: &Surface) -> Self {
        let physical_device = Self::pick_suitable_device(instance, surface);
        let queue_families = QueueFamily::all(instance, physical_device);
        let (graphics_index, present_index) = Self::get_queue_indices(instance, physical_device, surface).unwrap();


        PhysicalDevice {
            physical_device: physical_device,
            queue_families: queue_families,
            graphics_index: graphics_index,
            present_index: present_index
        }
    }

    fn pick_suitable_device(instance: &Instance, surface: &Surface) -> vk::PhysicalDevice {
        let physical_devices = unsafe { instance.enumerate_physical_devices().expect("Failed to enumerate physical devices") };
        debug!("{} devices (GPU) found with vulkan support.", physical_devices.len());

        let mut result = None;

        for &physical_device in physical_devices.iter() {
            if result.is_none() && Self::is_device_suitable(instance, physical_device, surface) {
                result = Some(physical_device)
            }
        }

        match result {
            None => panic!("No suitable physical device"),
            Some(device) => device
        }
    }

    fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice, surface: &Surface) -> bool {
        Self::get_queue_indices(instance, device, surface).is_ok()
    }

    fn get_queue_indices(instance: &Instance, device: vk::PhysicalDevice, surface: &Surface) -> Result<(u32, u32), &'static str> {
        let queue_families = QueueFamily::all(instance, device);

        let mut graphics_index = None;
        let mut present_index = None;
        for queue_family in queue_families.iter() {
            if queue_family.count > 0 {
                if graphics_index.is_none() && queue_family.flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_index = Some(queue_family.index);
                }

                if present_index.is_none() && unsafe { surface.get_loader().get_physical_device_surface_support(device, queue_family.index, *surface.vulkan_object()) } {
                   present_index = Some(queue_family.index)
                }
            }
        }

        match graphics_index {
            Some(g_index) => {
                match present_index {
                    Some(p_index) => Ok((g_index, p_index)),
                    None => Err("Present queue not present")
                }
            },
            None => Err("Graphics queue not present")
        }
    }

    pub fn graphics_index(&self) -> &u32 {
        &self.graphics_index
    }

    pub fn present_index(&self) -> &u32 {
        &self.present_index
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