use sketch::{models::Mesh, Renderer};

pub struct Model {
    pub mesh: Mesh,
}

impl Model {
    // TODO: Replace with builder pattern
    pub fn new(mesh: Mesh) -> Self {
        Self { mesh }
    }

    pub fn render(&self, renderer: &Renderer) {
        renderer.render_mesh(&self.mesh);
    }
}
