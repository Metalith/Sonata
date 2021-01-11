use specs::{Join, Read, ReadStorage, System, Write};

use imgui::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{
    event::{Event, WindowEvent},
    window::Window,
};

use crate::{render::GraphicContext, ControlData, DeltaTime, MouseState, Player, Renderable, Transform, WinitEventData};

pub struct RenderSystem {
    graphic_context: GraphicContext,
    imgui: Context,
    platform: WinitPlatform,
    imgui_renderer: imgui_rs_vulkan_renderer::Renderer,
    window: Window,
    window_focused: bool,

    curr_image_index: usize,

    camera_pos: uv::Vec3,
    camera_dir: uv::Vec3,
    camera_up: uv::Vec3,
}

impl RenderSystem {
    pub fn new(window: Window, graphic_context: GraphicContext) -> Self {
        let (mut imgui, platform) = Self::configure_imgui(&window);
        let imgui_renderer = imgui_rs_vulkan_renderer::Renderer::new(&graphic_context, 2, *graphic_context.get_render_pass(), &mut imgui).unwrap();

        RenderSystem {
            graphic_context,
            imgui,
            platform,
            imgui_renderer,
            window,
            window_focused: true,
            curr_image_index: 0,
            camera_pos: uv::Vec3::default(),
            camera_dir: uv::Vec3::default(),
            camera_up: uv::Vec3::default(),
        }
    }

    fn configure_imgui(window: &Window) -> (Context, WinitPlatform) {
        let mut imgui = Context::create();
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        }]);
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        (imgui, platform)
    }
}

impl<'a> System<'a> for RenderSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Read<'a, WinitEventData>,
        Read<'a, DeltaTime>,
        Write<'a, ControlData>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, (events_storage, delta_time, mut control_data, player_storage, transform_storage, render_storage): Self::SystemData) {
        let mut player_pos = uv::Vec3::default();
        let mut player_dir = uv::Rotor3::default();

        for (_, transform) in (&player_storage, &transform_storage).join() {
            player_pos = transform.pos;
            player_dir = transform.dir.normalized();
        }

        self.imgui.io_mut().update_delta_time(delta_time.delta);
        for event in &events_storage.events {
            self.platform.handle_event(self.imgui.io_mut(), &self.window, event);
            if let Event::WindowEvent { window_id: _, event } = event {
                if let WindowEvent::Focused(focused) = event {
                    self.window_focused = *focused;
                }
            };
        }

        let fps = self.imgui.io().framerate;
        let ui = self.imgui.frame();

        match control_data.mouse_state {
            MouseState::Ui => {
                ui.set_mouse_cursor(Some(imgui::MouseCursor::Arrow));
                self.window.set_cursor_visible(true);
            }
            MouseState::Fly if self.window_focused => {
                ui.set_mouse_cursor(None);
                self.window.set_cursor_visible(false);
            }
            _ => (),
        }

        if control_data.set_mouse {
            let pos = winit::dpi::PhysicalPosition::new(control_data.last_mouse_pos.0, control_data.last_mouse_pos.1);
            self.window.set_cursor_position(pos).unwrap();
            control_data.set_mouse = false;
        }

        imgui::Window::new(im_str!("Hello world")).build(&ui, || {
            ui.text(im_str!("Hello world!"));
            ui.text(im_str!("This...is...imgui-rs!"));
            ui.separator();
            ui.text(format!("Running for: {:.3} seconds", delta_time.start_time.elapsed().as_secs_f32()));
            ui.text(format!("Player position: {:.2?}", player_pos));
            ui.text(format!("Average {:.3} ms/frame ({:.1} FPS)", 1000f32 / fps, fps));
            let mouse_pos = ui.io().mouse_pos;
            ui.text(format!("Mouse Position: ({:.1},{:.1})", mouse_pos[0], mouse_pos[1]));
        });

        self.platform.prepare_render(&ui, &self.window);

        let draw_data = ui.render();

        self.camera_pos = player_pos;
        let mut camera_vecs = [uv::Vec3::new(0.0, 0.0, 1.0), uv::Vec3::new(1.0, 0.0, 0.0)];
        player_dir.rotate_vecs(&mut camera_vecs);
        self.camera_dir = camera_vecs[0];
        self.camera_up = camera_vecs[0].cross(camera_vecs[1]);

        self.graphic_context.sync_objects.wait_fence_current();

        if !self.graphic_context.get_window().is_window_visible() {
            return;
        }

        self.curr_image_index = match self.graphic_context.acquire_next_image() {
            Err(_) => {
                self.graphic_context.recreate_swapchain();
                return;
            }
            Ok(i) => i,
        };

        self.graphic_context.update_uniforms(self.curr_image_index, &self.camera_pos, &self.camera_dir, &self.camera_up);

        self.graphic_context.sync_objects.wait_fence_image(self.curr_image_index);

        self.graphic_context.begin_command_buffer(self.curr_image_index);

        for renderable in render_storage.join() {
            renderable
                .mesh
                .render(self.graphic_context.get_device(), self.graphic_context.get_command_buffer(self.curr_image_index));
        }
        self.imgui_renderer
            .cmd_draw(&self.graphic_context, *self.graphic_context.get_command_buffer(self.curr_image_index), draw_data)
            .unwrap();

        self.graphic_context.end_command_buffer(self.curr_image_index);

        self.graphic_context.submit_queue(self.curr_image_index);
        match self.graphic_context.present_queue(self.curr_image_index as u32) {
            Ok(_) => {}
            Err(_) => self.graphic_context.recreate_swapchain(),
        };

        self.graphic_context.sync_objects.increment_frame();
    }
}

impl Drop for RenderSystem {
    fn drop(&mut self) {
        trace!("Dropping Renderer");
        self.graphic_context.wait_device();
        self.imgui_renderer.destroy(&self.graphic_context).unwrap();
    }
}
