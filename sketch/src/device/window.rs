pub type HWND = *mut std::ffi::c_void;
pub type HINSTANCE = *mut std::ffi::c_void;

use std::cell::Cell;
use std::mem::zeroed;

pub struct Window {
    window_size: Cell<(u32, u32)>,
    hwnd: HWND,
}

impl Window {
    pub fn new(hwnd: HWND) -> Window {
        Window {
            window_size: Cell::new(get_window_size(hwnd)),
            hwnd: hwnd,
        }
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        get_window_size(self.hwnd)
    }

    pub fn is_window_visible(&self) -> bool {
        unsafe { (winapi::um::winuser::GetWindowLongA(self.hwnd as *mut winapi::shared::windef::HWND__, winapi::um::winuser::GWL_STYLE) as u32 & winapi::um::winuser::WS_MINIMIZE) == 0 }
    }

    pub fn has_window_resized(&self) -> bool {
        let res = self.window_size.get() != self.get_window_size();
        self.window_size.set(self.get_window_size());
        res
    }
}

fn get_window_size(hwnd: HWND) -> (u32, u32) {
    unsafe {
        let mut rect = zeroed::<winapi::shared::windef::RECT>();

        winapi::um::winuser::GetWindowRect(hwnd as *mut winapi::shared::windef::HWND__, &mut rect);

        ((rect.right - rect.left).abs() as u32, (rect.bottom - rect.top).abs() as u32)
    }
}
