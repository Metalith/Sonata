use wind::System;

pub struct RenderSystem {
    renderer: sketch::Renderer,
}

impl RenderSystem {
    pub fn new(win: &winit::window::Window) -> Self {
        let renderer = sketch::Renderer::new(win);
        renderer.setup();
        RenderSystem { renderer: renderer }
    }
}

impl System for RenderSystem {
    fn update(&self) {
        self.renderer.draw_frame();
    }
}
