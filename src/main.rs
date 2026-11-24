use std::num::NonZeroU32;
use std::rc::Rc;

use image::RgbaImage;
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    img: RgbaImage,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("see?");
        let window = Rc::new(event_loop.create_window(attrs).unwrap());

        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        window.request_redraw();
        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else {
                    return;
                };
                let size = window.inner_size();
                let (Some(w), Some(h)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                else {
                    return;
                };

                surface.resize(w, h).unwrap();
                let mut buffer = surface.buffer_mut().unwrap();

                let width = w.get();
                let height = h.get();

                let img_w = self.img.width();
                let img_h = self.img.height();

                let (draw_w, draw_h) = if img_w * height > img_h * width {
                    (width, img_h * width / img_w)
                } else {
                    (img_w * height / img_h, height)
                };

                let left = (width - draw_w) / 2;
                let top = (height - draw_h) / 2;
                for (i, px) in buffer.iter_mut().enumerate() {
                    let x = i as u32 % width;
                    let y = i as u32 / width;

                    *px = if x >= left && x < left + draw_w && y >= top && y < top + draw_h {
                        let src_x = (x - left) * img_w / draw_w;
                        let src_y = (y - top) * img_h / draw_h;

                        let [r, g, b, _a] = self.img.get_pixel(src_x, src_y).0;
                        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                    } else {
                        0x00181818
                    };
                }

                buffer.present().unwrap();
            }
            _ => {}
        }
    }
}

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: see <image path>");
        std::process::exit(1);
    };

    let img = match image::open(&path) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            eprintln!("could not open {path}: {e}");
            std::process::exit(1);
        }
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        window: None,
        surface: None,
        img,
    };
    event_loop.run_app(&mut app).unwrap();
}
