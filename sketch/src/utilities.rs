use std::{ffi::CStr, os::raw::c_char};

pub fn vk_to_str(vk_str: &[c_char]) -> &str {
    let raw_str = unsafe { CStr::from_ptr(vk_str.as_ptr()) };
    raw_str.to_str().unwrap()
}
