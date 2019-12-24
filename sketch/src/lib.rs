#[macro_use] extern crate log;

mod instance;
mod queue_family;
mod physical_device;
mod debug;
mod utility;

use instance::Instance;
use debug::DebugMessenger;
use physical_device::PhysicalDevice;

use winit::window::Window;
use ash::Entry;


pub struct Renderer {
    entry: Entry,
    instance: Instance,
    physical_device: PhysicalDevice
}

impl Renderer {
    pub fn new(win : &Window) -> Self {
        let entry = Entry::new().unwrap();
        let instance = Instance::new(win, &entry);
        let physical_device = PhysicalDevice::new(&instance.vulkan_object());

        Renderer {
            entry: entry,
            instance : instance,
            physical_device: physical_device
        }
    }
}

impl Drop for Renderer {
    fn drop (&mut self) {
        self.instance.cleanup();
    }
}

trait VulkanObject {
    type Object;
    fn vulkan_object(&self) -> &Self::Object;
}