use super::{QueueFamily, Surface};
use crate::{renderpasses::SwapChain, utilities::utility, GraphicContext, VulkanObject};

use ash::{extensions::khr::Swapchain, version::InstanceV1_0, vk, Instance};

use std::ffi::CStr;

pub struct PhysicalDevice {
    physical_device: vk::PhysicalDevice,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
    graphics_index: u32,
    present_index: u32,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, surface: &Surface) -> Self {
        let physical_device = Self::pick_suitable_device(instance, surface);
        let (graphics_index, present_index) = Self::get_queue_indices(instance, physical_device, surface).unwrap();
        let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

        PhysicalDevice {
            physical_device,
            mem_properties,
            graphics_index,
            present_index,
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
            Some(device) => device,
        }
    }

    fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice, surface: &Surface) -> bool {
        let extensions_supported = Self::check_device_extension_support(instance, device);
        let mut swapchain_adequate = false;
        if extensions_supported {
            let swapchain_support = SwapChain::query_support(device, surface);
            swapchain_adequate = !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty();
        }

        Self::get_queue_indices(instance, device, surface).is_ok() && extensions_supported && swapchain_adequate
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

                if present_index.is_none() && unsafe { surface.get_loader().get_physical_device_surface_support(device, queue_family.index, *surface.vulkan_object()).unwrap() } {
                    present_index = Some(queue_family.index)
                }
            }
        }

        match graphics_index {
            Some(g_index) => match present_index {
                Some(p_index) => Ok((g_index, p_index)),
                None => Err("Present queue not present"),
            },
            None => Err("Graphics queue not present"),
        }
    }

    fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        let available_extensions = unsafe { instance.enumerate_device_extension_properties(device).unwrap() };
        let required_extensions: Vec<&str> = Self::required_extension_names().iter().map(|name| unsafe { CStr::from_ptr(*name).to_str().unwrap() }).collect();

        debug!("Device Extensions Available:");
        for extension in available_extensions.iter() {
            let name = utility::vk_to_str(&extension.extension_name);
            debug!("\t{}", name);
        }

        let mut result = true;

        for &required_extension_name in required_extensions.iter() {
            if result {
                result = available_extensions.iter().any(|extension| utility::vk_to_str(&extension.extension_name) == required_extension_name)
            }
        }

        result
    }

    pub fn required_extension_names() -> Vec<*const i8> {
        vec![Swapchain::name().as_ptr()]
    }

    pub fn graphics_index(&self) -> &u32 {
        &self.graphics_index
    }

    pub fn present_index(&self) -> &u32 {
        &self.present_index
    }

    pub fn get_mem_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.mem_properties
    }
}

impl VulkanObject for PhysicalDevice {
    type Object = vk::PhysicalDevice;

    fn vulkan_object(&self) -> &Self::Object {
        &self.physical_device
    }

    fn cleanup(&self, _context: &GraphicContext) {}
}
