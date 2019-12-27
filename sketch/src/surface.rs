use crate::VulkanObject; 

use ash::{
    vk,
    Instance,
    Entry,
    extensions::khr
};
use winit::window::Window;
use winit::platform::windows::WindowExtWindows;

pub struct Surface {
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR
}

impl Surface {
    pub fn new(window: &Window, entry: &Entry, instance: &Instance) -> Self {
        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hwnd(window.hwnd())
            .hinstance(window.hinstance())
            .build();

        let win32_surface_loader = khr::Win32Surface::new(entry, instance);
        let surface = unsafe { win32_surface_loader.create_win32_surface(&create_info, None).expect("Failed to create a window surface") };

        Surface {
            surface_loader: khr::Surface::new(entry, instance),
            surface: surface
        }
    }

    pub fn get_loader(&self) -> &khr::Surface {
        &self.surface_loader
    }
}

impl VulkanObject for Surface {
    type Object = vk::SurfaceKHR;

    fn vulkan_object(&self) -> &vk::SurfaceKHR {
        &self.surface
    }

    fn cleanup (&self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}