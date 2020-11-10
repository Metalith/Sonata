use specs::{Join, Read, ReadStorage, System, Write};

use imgui::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{
    event::{Event, WindowEvent},
    window::Window,
};

use crate::{ControlData, DeltaTime, MouseState, Player, Renderable, Transform, WinitEventData};

pub struct RenderSystem {
    renderer: sketch::Renderer,
    win: Window,
    imgui: Context,
    platform: WinitPlatform,
    window_focused: bool,
}

impl RenderSystem {
    pub fn new(window: Window, renderer: sketch::Renderer) -> Self {
        let mut renderer = renderer;
        let (mut imgui, platform) = Self::configure_imgui(&window);
        renderer.add_imgui_renderer(&mut imgui);

        RenderSystem {
            renderer,
            win: window,
            imgui,
            platform,
            window_focused: true,
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
            self.platform.handle_event(self.imgui.io_mut(), &self.win, event);
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
                self.win.set_cursor_visible(true);
            }
            MouseState::Fly if self.window_focused => {
                ui.set_mouse_cursor(None);
                self.win.set_cursor_visible(false);
            }
            _ => (),
        }

        if control_data.set_mouse {
            let pos = winit::dpi::PhysicalPosition::new(control_data.last_mouse_pos.0, control_data.last_mouse_pos.1);
            self.win.set_cursor_position(pos).unwrap();
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

        self.platform.prepare_render(&ui, &self.win);

        let draw_data = ui.render();

        let camera_pos = player_pos;
        let mut camera_vecs = [uv::Vec3::new(0.0, 0.0, 1.0), uv::Vec3::new(1.0, 0.0, 0.0)];
        player_dir.rotate_vecs(&mut camera_vecs);
        let camera_up = camera_vecs[0].cross(camera_vecs[1]);
        self.renderer.update_camera(&camera_pos.into(), &camera_vecs[0].into(), &camera_up.into());
        if self.renderer.begin_frame() {
            for renderable in render_storage.join() {
                self.renderer.render_mesh(&renderable.mesh);
            }
            self.renderer.draw_imgui(draw_data);
            self.renderer.end_frame();
        }
    }
}
