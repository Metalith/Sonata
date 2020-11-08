mod debug;
mod instance;
mod physical_device;
mod queue_family;
mod surface;
pub mod window;

pub use debug::DebugMessenger;
pub use instance::Instance;
pub use physical_device::PhysicalDevice;
pub use queue_family::QueueFamily;
pub use surface::Surface;
pub use window::Window;

use crate::{commands::CommandPool, pipelines::DescriptorPool, VulkanObject};

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};

use std::{
    collections::HashSet,
    sync::{Arc, Mutex, Weak},
};

pub struct Device {
    physical_device: PhysicalDevice,
    logical_device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    command_pool: Mutex<Weak<CommandPool>>,
    descriptor_pool: Mutex<Weak<DescriptorPool>>,
}

impl Device {
    pub fn new(physical_device: PhysicalDevice, validation: bool) -> Arc<Self> {
        let queue_families: HashSet<u32> = vec![physical_device.graphics_index(), physical_device.present_index()].into_iter().collect();
        let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();

        let priority = [1f32];
        for &queue_index in queue_families.iter() {
            queue_create_infos.push(vk::DeviceQueueCreateInfo::builder().queue_family_index(queue_index).queue_priorities(&priority).build());
        }

        let device_features = vk::PhysicalDeviceFeatures::default();
        let extensions = PhysicalDevice::required_extension_names();
        let (_names, validation_layers) = DebugMessenger::get_validation_layers_vk();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(if validation { &validation_layers } else { &[] })
            .build();

        let device: ash::Device = unsafe {
            physical_device
                .instance()
                .vk()
                .create_device(*physical_device.vk(), &create_info, None)
                .expect("Failed to create logical device")
        };
        let graphics_queue = unsafe { device.get_device_queue(physical_device.graphics_index(), 0) };
        let present_queue = unsafe { device.get_device_queue(physical_device.present_index(), 0) };

        Arc::new(Device {
            physical_device,
            logical_device: device,
            graphics_queue,
            present_queue,
            command_pool: Mutex::new(Weak::new()),
            descriptor_pool: Mutex::new(Weak::new()),
        })
    }

    pub fn graphics_queue(&self) -> &vk::Queue {
        &self.graphics_queue
    }

    pub fn present_queue(&self) -> &vk::Queue {
        &self.present_queue
    }

    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn instance(&self) -> &Arc<Instance> {
        self.physical_device.instance()
    }

    pub fn command_pool(self: &Arc<Self>) -> Arc<CommandPool> {
        let mut command_pool = self.command_pool.lock().unwrap();

        if let Some(pool) = command_pool.upgrade() {
            pool
        } else {
            let new_pool = CommandPool::new(self.clone());
            *command_pool = Arc::downgrade(&new_pool);
            new_pool
        }
    }

    pub fn descriptor_pool(self: &Arc<Self>) -> Arc<DescriptorPool> {
        let mut descriptor_pool = self.descriptor_pool.lock().unwrap();

        if let Some(pool) = descriptor_pool.upgrade() {
            pool
        } else {
            let new_pool = Arc::new(DescriptorPool::new(self.clone()));
            *descriptor_pool = Arc::downgrade(&new_pool);
            new_pool
        }
    }
}

impl VulkanObject for Device {
    type Object = ash::Device;

    fn vk(&self) -> &Self::Object {
        &self.logical_device
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        trace!("Dropping Device");
        unsafe {
            self.logical_device.destroy_device(None);
        }
    }
}
