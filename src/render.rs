use wind::System;

pub struct RenderSystem {
    renderer: sketch::Renderer,
}

impl RenderSystem {
    pub fn new(win: &winit::window::Window) -> Self {
        RenderSystem { renderer: sketch::Renderer::new(win) }
    }
}

impl System for RenderSystem {
    fn update(&self) {
        self.renderer.draw_frame();
    }
}
