use wind::System;
use winit::window::Window;

use std::cell::RefCell;

pub struct RenderSystem<'a> {
    pub renderer: RefCell<sketch::Renderer<'a>>,
}

impl<'a> RenderSystem<'a> {
    pub fn new(win: &'a Window) -> Self {
        let b = Box::new(move || -> (u32, u32) {
            let t = win.outer_size();
            (t.width, t.height)
        });
        let renderer = sketch::Renderer::new(win, b);
        renderer.setup();
        RenderSystem { renderer: RefCell::new(renderer) }
    }
}

impl<'a> System for RenderSystem<'a> {
    fn update(&self) {
        self.renderer.borrow_mut().draw_frame();
    }
}
