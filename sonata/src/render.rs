use specs::{Join, Read, ReadStorage, System};

use sketch::model::Vertex;

use imgui::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{platform::windows::WindowExtWindows, window::Window};

use std::cell::RefCell;

use crate::{DeltaTime, Player, Transform, WinitEventData};

pub struct RenderSystem {
    pub renderer: RefCell<sketch::Renderer>,
    win: Window,
    pub imgui: RefCell<Context>,
    platform: WinitPlatform,
}

impl RenderSystem {
    pub fn new(win: Window) -> Self {
        let mut imgui = Context::create();
        // configure imgui-rs Context if necessary

        let mut platform = WinitPlatform::init(&mut imgui); // step 1
        platform.attach_window(imgui.io_mut(), &win, HiDpiMode::Default); // step 2

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        }]);
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        let vertices = [
            Vertex {
                pos: [-0.5f32, -0.5f32],
                color: [1.0f32, 0.0f32, 0.0f32],
            },
            Vertex {
                pos: [0.5f32, -0.5f32],
                color: [0.0f32, 1.0f32, 0.0f32],
            },
            Vertex {
                pos: [0.5f32, 0.5f32],
                color: [0.0f32, 0.0f32, 1.0f32],
            },
            Vertex {
                pos: [-0.5f32, 0.5f32],
                color: [1.0f32, 1.0f32, 1.0f32],
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let vertices2 = [
            Vertex {
                pos: [-1f32, -1f32],
                color: [1.0f32, 0.0f32, 0.0f32],
            },
            Vertex {
                pos: [-0.5f32, -1f32],
                color: [1.0f32, 1.0f32, 1.0f32],
            },
            Vertex {
                pos: [-1f32, -0.5f32],
                color: [0.0f32, 0.0f32, 1.0f32],
            },
        ];

        let mut renderer = sketch::Renderer::new(win.hwnd(), win.hinstance());

        renderer.add_imgui_renderer(&mut imgui);
        renderer.add_model(&vertices, Some(&indices));
        renderer.add_model(&vertices2, None);

        RenderSystem {
            renderer: RefCell::new(renderer),
            win,
            imgui: RefCell::new(imgui),
            platform,
        }
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (Read<'a, WinitEventData>, Read<'a, DeltaTime>, ReadStorage<'a, Player>, ReadStorage<'a, Transform>);

    fn run(&mut self, (events_storage, delta_time, player_storage, transform_storage): Self::SystemData) {
        let mut player_pos = [0.0, 0.0, 0.0];
        for (_, transform) in (&player_storage, &transform_storage).join() {
            player_pos = transform.pos;
        }

        let mut imgui = self.imgui.borrow_mut();

        imgui.io_mut().update_delta_time(delta_time.last_frame);
        for event in &events_storage.events {
            self.platform.handle_event(imgui.io_mut(), &self.win, event);
        }

        let fps = imgui.io().framerate;
        let ui = imgui.frame();

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

        self.platform.prepare_render(&ui, &self.win); // step 5
        let draw_data = ui.render();

        self.renderer.borrow_mut().draw_frame(Some(draw_data));
    }
}
