use crate::utility;
use ash::{
    vk,
    Entry,
    Instance,
    version::EntryV1_0
};

use std::os::raw::c_void;
use std::ffi::CStr;

pub struct DebugMessenger {
    debug_messenger : vk::DebugUtilsMessengerEXT,
    debug_loader : ash::extensions::ext::DebugUtils,
    validation_enabled: bool
}

impl DebugMessenger {
    pub fn new(entry : &Entry, instance: &Instance) -> Self {
        // TODO: Disable this module if not debugging
        let loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        let create_info = Self::populate_debug_messenger_create_info();

        let env_enable = std::env::var("WIND_VK_VALIDATION").is_ok();
        let validation_enabled: bool  = if env_enable { std::env::var("WIND_VK_VALIDATION").unwrap().parse().unwrap() } else { false };

        let utils_messenger = unsafe {
            if validation_enabled {
                loader.create_debug_utils_messenger(&create_info, None).expect("Failed to create debug messenger`")
            } else {
                ash::vk::DebugUtilsMessengerEXT::null()
            }
        };

        DebugMessenger {
            debug_messenger: utils_messenger,
            debug_loader: loader,
            validation_enabled: validation_enabled
        }
    }

    pub fn check_validation_layer_support(entry : &Entry) -> bool {
        let available_layers = entry.enumerate_instance_layer_properties().unwrap();
        let required_layers = Self::get_validation_layers();

        if available_layers.len() <= 0 {
            error!("No available layers.");
            return false;
        } else {
            debug!("Instance Available Layers:");
            for layer in available_layers.iter() {
                let name = utility::vk_to_str(&layer.layer_name);
                debug!("\t{}", name);
            }
        }

        for required_layer_name in required_layers {
            let mut layer_found = false;

            for layer in available_layers.iter() {
                let layer_name =  utility::vk_to_str(&layer.layer_name);
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

    pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity : vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            message_type : vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            ..Default::default()
        }
    }

    pub unsafe fn cleanup(&self) {
        if self.validation_enabled {
            self.debug_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
        }
    }
}

unsafe extern "system"
fn vulkan_debug_utils_callback(
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