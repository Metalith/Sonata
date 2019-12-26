#[macro_use] extern crate log;

mod instance;
mod queue_family;
mod physical_device;
mod logical_device;
mod debug;
mod utility;

use instance::Instance;
use debug::DebugMessenger;
use physical_device::PhysicalDevice;
use logical_device::LogicalDevice;

use winit::window::Window;
use ash::Entry;


pub struct Renderer {
    entry: Entry,
    instance: Instance,
    physical_device: PhysicalDevice,
    logical_device: LogicalDevice,
}

impl Renderer {
    pub fn new(win : &Window) -> Self {
        let entry = Entry::new().unwrap();
        let instance = Instance::new(win, &entry);
        let physical_device = PhysicalDevice::new(instance.vulkan_object());
        let logical_device = LogicalDevice::new(instance.vulkan_object(), &physical_device);

        Renderer {
            entry: entry,
            instance : instance,
            physical_device: physical_device,
            logical_device: logical_device
        }
    }
}

impl Drop for Renderer {
    fn drop (&mut self) {
        self.logical_device.cleanup();
        self.instance.cleanup();
    }
}

trait VulkanObject {
    type Object;
    fn vulkan_object(&self) -> &Self::Object;
    fn cleanup(&self);
}