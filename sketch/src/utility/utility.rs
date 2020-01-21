use std::ffi::CStr;
use std::os::raw::c_char;

pub fn vk_to_str(vk_str: &[c_char]) -> &str {
    let raw_str = unsafe { CStr::from_ptr(vk_str.as_ptr()) };
    raw_str.to_str().unwrap()
}

pub fn validation_enabled() -> bool {
    if std::env::var("WIND_VK_VALIDATION").is_ok() {
        std::env::var("WIND_VK_VALIDATION").unwrap().parse::<bool>().unwrap()
    } else {
        false
    }
}
