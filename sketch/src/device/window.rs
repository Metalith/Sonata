pub type HWND = *mut std::ffi::c_void;
pub type HINSTANCE = *mut std::ffi::c_void;

use std::cell::Cell;

pub struct Window<'a> {
    window_size: Cell<(u32, u32)>,
    window_size_cb: Box<dyn Fn() -> (u32, u32) + 'a>,
}

impl<'a> Window<'a> {
    pub fn new<T: Fn() -> (u32, u32) + 'a>(window_size_cb: T) -> Window<'a> {
        Window {
            window_size: Cell::new(window_size_cb()),
            window_size_cb: Box::new(window_size_cb),
        }
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        (*self.window_size_cb)()
    }

    pub fn window_is_minimized(&self) -> bool {
        self.get_window_size() == (0, 0)
    }

    pub fn has_window_resized(&self) -> bool {
        let res = self.window_size.get() != self.get_window_size();
        self.window_size.set(self.get_window_size());
        res
    }
}
