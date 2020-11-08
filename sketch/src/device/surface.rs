use std::sync::Arc;

use crate::{
    device::window::{HINSTANCE, HWND},
    VulkanObject,
};

use ash::{extensions::khr, vk};

use super::Instance;

pub struct Surface {
    _instance: Arc<Instance>,
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(hwnd: HWND, hinstance: HINSTANCE, instance: Arc<Instance>) -> Arc<Self> {
        let create_info = vk::Win32SurfaceCreateInfoKHR::builder().hwnd(hwnd).hinstance(hinstance).build();

        let surface_loader = khr::Surface::new(instance.entry(), instance.vk());
        let surface = unsafe {
            khr::Win32Surface::new(instance.entry(), instance.vk())
                .create_win32_surface(&create_info, None)
                .expect("Failed to create a window surface")
        };

        Arc::new(Surface {
            _instance: instance,
            surface_loader,
            surface,
        })
    }

    pub fn get_loader(&self) -> &khr::Surface {
        &self.surface_loader
    }
}

impl VulkanObject for Surface {
    type Object = vk::SurfaceKHR;

    fn vk(&self) -> &Self::Object {
        &self.surface
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        trace!("Dropping Surface");
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
