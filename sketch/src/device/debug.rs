use crate::{utilities::vk_to_str, VulkanObject};

use ash::{version::EntryV1_0, vk, Entry};

use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
    sync::Arc,
};

use super::Instance;

pub struct DebugMessenger {
    _instance: Arc<Instance>,

    debug_messenger: vk::DebugUtilsMessengerEXT,
    debug_loader: ash::extensions::ext::DebugUtils,
}

impl DebugMessenger {
    pub fn new(instance: &Arc<Instance>) -> Arc<Self> {
        let loader = ash::extensions::ext::DebugUtils::new(instance.entry(), instance.vk());

        let create_info = Self::populate_debug_messenger_create_info();

        let utils_messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None).expect("Failed to create debug messenger`") };

        Arc::new(DebugMessenger {
            debug_messenger: utils_messenger,
            debug_loader: loader,
            _instance: instance.clone(),
        })
    }

    pub fn check_validation_layer_support(entry: &Entry) -> bool {
        let available_layers = entry.enumerate_instance_layer_properties().unwrap();
        let required_layers = Self::get_validation_layers();

        if available_layers.is_empty() {
            error!("No available layers.");
            return false;
        } else {
            debug!("Instance Available Layers:");
            for layer in available_layers.iter() {
                let name = vk_to_str(&layer.layer_name);
                debug!("\t{}", name);
            }
        }

        for required_layer_name in required_layers {
            let mut layer_found = false;

            for layer in available_layers.iter() {
                let layer_name = vk_to_str(&layer.layer_name);
                if required_layer_name == layer_name {
                    layer_found = true;
                }
            }

            if !layer_found {
                return false;
            }
        }

        true
    }

    pub fn get_validation_layers() -> Vec<&'static str> {
        vec!["VK_LAYER_KHRONOS_validation"]
    }

    pub fn get_validation_layers_vk() -> (Vec<CString>, Vec<*const i8>) {
        let layers = Self::get_validation_layers();

        let layer_names = layers.iter().map(|layer_name| CString::new(*layer_name).expect("Failed to build CString")).collect::<Vec<CString>>();

        let layer_ptrs: Vec<*const i8> = layer_names.iter().map(|layer_name| layer_name.as_ptr()).collect();

        (layer_names, layer_ptrs)
    }

    pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            ..Default::default()
        }
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        trace!("Dropping Debug Messenger");
        unsafe {
            self.debug_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
        }
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    debug!("{}{}{:?}", severity, types, message);

    vk::FALSE
}
