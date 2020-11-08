use crate::VulkanObject;

use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Win32Surface},
    },
    version::{EntryV1_0, InstanceV1_0},
    vk, Entry,
};

use std::{ffi::CString, os::raw::c_void, ptr, sync::Arc};

use super::DebugMessenger;

pub struct Instance {
    entry: Entry,
    instance: ash::Instance,
}

impl Instance {
    pub fn new(validation: bool) -> Arc<Self> {
        let entry = Entry::new().unwrap();

        if validation && !DebugMessenger::check_validation_layer_support(&entry) {
            panic!("Validation layers requested not supported");
        }

        let app_name = CString::new("Hello world").unwrap(); // Generate this somewhere
        let engine_name = CString::new("No engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_version(0, 0, 1))
            .engine_name(&engine_name)
            .engine_version(vk::make_version(0, 0, 1))
            .api_version(vk::make_version(1, 1, 106))
            .build();

        let (_names, validation_layers) = DebugMessenger::get_validation_layers_vk();

        let mut extensions = Self::required_extension_names();
        if validation {
            extensions.push(DebugUtils::name().as_ptr());
        }

        let debug_create_info = DebugMessenger::populate_debug_messenger_create_info();

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            p_next: if validation {
                &debug_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
            } else {
                ptr::null()
            },
            pp_enabled_layer_names: if validation { validation_layers.as_ptr() } else { ptr::null() },
            enabled_layer_count: if validation { validation_layers.len() } else { 0 } as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            ..Default::default()
        };

        let instance: ash::Instance = unsafe { entry.create_instance(&create_info, None).expect("Failed to create instance") };

        Arc::new(Instance { entry, instance })
    }

    fn required_extension_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()]
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}

impl VulkanObject for Instance {
    type Object = ash::Instance;

    fn vk(&self) -> &Self::Object {
        &self.instance
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        trace!("Dropping Instance");
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
