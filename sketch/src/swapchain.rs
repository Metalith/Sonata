use crate::VulkanObject; 

pub struct Swapchain {

}

impl Swapchain {

}

impl VulkanObject for Swapchain {
    type Object = ash::Instance;

    fn vulkan_object(&self) -> &ash::Instance {
        &self.instance
    }
}