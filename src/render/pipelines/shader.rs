use std::sync::Arc;

use ash::{version::DeviceV1_0, vk};

use crate::render::{device::Device, VulkanObject};

// Returns a shader module from the source file
pub fn create_shader_module(file_name: &str, device: &Arc<Device>) -> Result<vk::ShaderModule, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(file_name)?;
    let words = ash::util::read_spv(&mut file).unwrap();

    let create_info = vk::ShaderModuleCreateInfo::builder().code(&words).build();

    let shader_module = unsafe { device.vk().create_shader_module(&create_info, None).unwrap() };
    Ok(shader_module)
}
