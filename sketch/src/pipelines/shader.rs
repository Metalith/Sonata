use ash::{version::DeviceV1_0, vk, Device};

// Returns a shader module from the source file
pub fn create_shader_module(file_name: &str, device: &Device) -> Result<vk::ShaderModule, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(file_name)?;
    let words = ash::util::read_spv(&mut file).unwrap();

    let create_info = vk::ShaderModuleCreateInfo::builder().code(&words).build();

    let shader_module = unsafe { device.create_shader_module(&create_info, None).unwrap() };
    Ok(shader_module)
}
