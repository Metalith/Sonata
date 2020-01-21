use wind::System;

use sketch::model::Vertex;

use winit::window::Window;

use std::cell::RefCell;

pub struct RenderSystem<'a> {
    pub renderer: RefCell<sketch::Renderer<'a>>,
}

impl<'a> RenderSystem<'a> {
    pub fn new(win: &'a Window) -> Self {
        let b = Box::new(move || -> (u32, u32) {
            let t = win.inner_size();
            (t.width, t.height)
        });

        let vertices = [
            Vertex {
                pos: [0.0f32, -0.5f32],
                color: [1.0f32, 0.0f32, 0.0f32],
            },
            Vertex {
                pos: [0.5f32, 0.5f32],
                color: [0.0f32, 1.0f32, 0.0f32],
            },
            Vertex {
                pos: [-0.5f32, 0.5f32],
                color: [0.0f32, 0.0f32, 1.0f32],
            },
        ];
        let renderer = sketch::Renderer::new(win, b, &vertices);
        renderer.setup();
        RenderSystem { renderer: RefCell::new(renderer) }
    }
}

impl<'a> System for RenderSystem<'a> {
    fn update(&self) {
        self.renderer.borrow_mut().draw_frame();
    }
}
