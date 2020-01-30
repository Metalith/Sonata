use crate::device::window::{HINSTANCE, HWND};
use crate::graphic_context::GraphicContext;
use crate::model::Model;
use crate::model::Vertex;

use std::time::Instant;

pub struct Renderer<'a> {
    graphic_context: GraphicContext<'a>,
    models: Vec<Model>,
    fps_timer: Instant,
    fps_counter: u32,
}

impl<'a> Renderer<'a> {
    pub fn new<T: Fn() -> (u32, u32) + 'a>(hwnd: HWND, hinstance: HINSTANCE, window_size_cb: T) -> Renderer<'a> {
        let graphic_context = GraphicContext::new(hwnd, hinstance, window_size_cb);
        Renderer {
            graphic_context: graphic_context,
            models: Vec::new(),
            fps_timer: Instant::now(),
            fps_counter: 0,
        }
    }

    pub fn add_model(&mut self, vertices: &[Vertex], indices: Option<&[u16]>) {
        self.models.push(self.graphic_context.create_model(vertices, indices));
    }

    pub fn draw_frame(&mut self) {
        self.graphic_context.sync_objects.wait_fence_current(self.graphic_context.get_device());

        if self.graphic_context.get_window().window_is_minimized() {
            return;
        }

        let image_index = match self.graphic_context.acquire_next_image() {
            Err(_) => {
                self.graphic_context.recreate_swapchain();
                return;
            }
            Ok(i) => i,
        };

        self.graphic_context.sync_objects.wait_fence_image(self.graphic_context.get_device(), image_index);

        self.graphic_context.begin_command_buffer(image_index);
        self.graphic_context.render_models(image_index, &self.models);
        self.graphic_context.end_command_buffer(image_index);

        self.graphic_context.submit_queue(image_index);
        match self.graphic_context.present_queue(image_index as u32) {
            Ok(_) => {}
            Err(_) => self.graphic_context.recreate_swapchain(),
        };

        self.graphic_context.sync_objects.increment_frame();
        self.fps_counter += 1;

        let new_now = Instant::now();
        if new_now.duration_since(self.fps_timer).as_secs() > 0 {
            debug!("FPS: {:?}", self.fps_counter);
            self.fps_timer = new_now;
            self.fps_counter = 0;
        }
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        self.graphic_context.wait_device();

        for model in self.models.iter() {
            model.cleanup(&self.graphic_context)
        }

        self.graphic_context.cleanup();
    }
}
