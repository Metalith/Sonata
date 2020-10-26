use crate::{
    device::window::{HINSTANCE, HWND},
    graphic_context::GraphicContext,
    models::{Mesh, Vertex},
};

pub struct Renderer {
    graphic_context: GraphicContext,
    camera_pos: uv::Vec3,
    camera_dir: uv::Vec3,
    camera_up: uv::Vec3,
    imgui_renderer: Option<imgui_rs_vulkan_renderer::Renderer>,
    curr_image_index: usize,
}

impl Renderer {
    pub fn new(hwnd: HWND, hinstance: HINSTANCE) -> Renderer {
        let graphic_context = GraphicContext::new(hwnd, hinstance);
        Renderer {
            graphic_context,
            camera_pos: uv::Vec3::new(2.0, 2.0, 2.0),
            camera_dir: uv::Vec3::new(-2.0, -2.0, -2.0),
            camera_up: uv::Vec3::new(0.0, 0.0, 1.0),
            imgui_renderer: None,
            curr_image_index: 0,
        }
    }

    pub fn add_imgui_renderer(&mut self, imgui: &mut imgui::Context) {
        self.imgui_renderer = Some(imgui_rs_vulkan_renderer::Renderer::new(&self.graphic_context, 2, *self.graphic_context.get_render_pass(), imgui).unwrap());
    }

    pub fn update_camera(&mut self, pos: &[f32; 3], dir: &[f32; 3], up: &[f32; 3]) {
        self.camera_pos = uv::Vec3::from(*pos);
        self.camera_dir = uv::Vec3::from(*dir);
        self.camera_up = uv::Vec3::from(*up);
    }

    pub fn begin_frame(&mut self) -> bool {
        self.graphic_context.sync_objects.wait_fence_current(self.graphic_context.get_device());

        if !self.graphic_context.get_window().is_window_visible() {
            return false;
        }

        self.curr_image_index = match self.graphic_context.acquire_next_image() {
            Err(_) => {
                self.graphic_context.recreate_swapchain();
                return false;
            }
            Ok(i) => i,
        };

        self.graphic_context.update_uniforms(self.curr_image_index, &self.camera_pos, &self.camera_dir, &self.camera_up);

        self.graphic_context.sync_objects.wait_fence_image(self.graphic_context.get_device(), self.curr_image_index);

        self.graphic_context.begin_command_buffer(self.curr_image_index);
        true
    }

    pub fn draw_imgui(&mut self, draw_data: &imgui::DrawData) {
        if let Some(imgui_renderer) = &mut self.imgui_renderer {
            imgui_renderer
                .cmd_draw(&self.graphic_context, *self.graphic_context.get_command_buffer(self.curr_image_index), draw_data)
                .unwrap();
        }
    }

    pub fn end_frame(&mut self) {
        self.graphic_context.end_command_buffer(self.curr_image_index);

        self.graphic_context.submit_queue(self.curr_image_index);
        match self.graphic_context.present_queue(self.curr_image_index as u32) {
            Ok(_) => {}
            Err(_) => self.graphic_context.recreate_swapchain(),
        };

        self.graphic_context.sync_objects.increment_frame();
    }

    pub fn create_mesh(&self, vertices: &[Vertex], indices: Option<&[u16]>) -> Mesh {
        Mesh::new(vertices, indices, &self.graphic_context)
    }

    pub fn render_mesh(&self, mesh: &Mesh) {
        mesh.render(self.graphic_context.get_device(), self.graphic_context.get_command_buffer(self.curr_image_index));
    }

    pub fn cleanup_mesh(&self, mesh: &Mesh) {
        self.graphic_context.wait_device();
        mesh.cleanup(&self.graphic_context);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphic_context.wait_device();

        if let Some(imgui_renderer) = &mut self.imgui_renderer {
            imgui_renderer.destroy(&self.graphic_context).unwrap();
        }

        self.graphic_context.cleanup();
    }
}
