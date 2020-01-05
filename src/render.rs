use wind::System;

use std::cell::RefCell;

pub struct RenderSystem {
    renderer: RefCell<sketch::Renderer>,
}

impl RenderSystem {
    pub fn new(win: &winit::window::Window) -> Self {
        let renderer = sketch::Renderer::new(win);
        renderer.setup();
        RenderSystem { renderer: RefCell::new(renderer) }
    }
}

impl System for RenderSystem {
    fn update(&self) {
        self.renderer.borrow_mut().draw_frame();
    }
}
