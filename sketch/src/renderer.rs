use crate::{
    device::window::{HINSTANCE, HWND},
    graphic_context::GraphicContext,
    models::{Model, Vertex},
};

use cgmath::Point3;

pub struct Renderer {
    graphic_context: GraphicContext,
    models: Vec<Model>,
    camera: Point3<f32>,
    imgui_renderer: Option<imgui_rs_vulkan_renderer::Renderer>,
}

impl Renderer {
    pub fn new(hwnd: HWND, hinstance: HINSTANCE) -> Renderer {
        let graphic_context = GraphicContext::new(hwnd, hinstance);
        Renderer {
            graphic_context,
            models: Vec::new(),
            camera: Point3::new(0.0, 0.0, 0.0),
            imgui_renderer: None,
        }
    }

    pub fn add_imgui_renderer(&mut self, imgui: &mut imgui::Context) {
        self.imgui_renderer = Some(imgui_rs_vulkan_renderer::Renderer::new(&self.graphic_context, 2, *self.graphic_context.get_render_pass(), imgui).unwrap());
    }

    pub fn add_model(&mut self, vertices: &[Vertex], indices: Option<&[u16]>) {
        self.models.push(self.graphic_context.create_model(vertices, indices));
    }

    pub fn update_camera(&mut self, pos: &[f32; 3]) {
        self.camera = Point3::from(*pos);
    }

    pub fn draw_frame(&mut self, imgui_draw_data: Option<&imgui::DrawData>) {
        self.graphic_context.sync_objects.wait_fence_current(self.graphic_context.get_device());

        if !self.graphic_context.get_window().is_window_visible() {
            return;
        }

        let image_index = match self.graphic_context.acquire_next_image() {
            Err(_) => {
                self.graphic_context.recreate_swapchain();
                return;
            }
            Ok(i) => i,
        };

        self.graphic_context.update_uniforms(image_index, &self.camera);

        self.graphic_context.sync_objects.wait_fence_image(self.graphic_context.get_device(), image_index);

        self.graphic_context.begin_command_buffer(image_index);
        self.graphic_context.render_models(image_index, &self.models);
        if let Some(imgui_renderer) = &mut self.imgui_renderer {
            if let Some(data) = imgui_draw_data {
                imgui_renderer.cmd_draw(&self.graphic_context, *self.graphic_context.get_command_buffer(image_index), data).unwrap();
            }
        }
        self.graphic_context.end_command_buffer(image_index);

        self.graphic_context.submit_queue(image_index);
        match self.graphic_context.present_queue(image_index as u32) {
            Ok(_) => {}
            Err(_) => self.graphic_context.recreate_swapchain(),
        };

        self.graphic_context.sync_objects.increment_frame();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graphic_context.wait_device();

        if let Some(imgui_renderer) = &mut self.imgui_renderer {
            imgui_renderer.destroy(&self.graphic_context).unwrap();
        }

        for model in self.models.iter() {
            model.cleanup(&self.graphic_context)
        }

        self.graphic_context.cleanup();
    }
}
