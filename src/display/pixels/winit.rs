use crate::display::pixels::drawer::Drawer;
use pixels::wgpu::PresentMode;
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::thread::JoinHandle;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{Window, WindowId};

pub fn create_pixels_window(
    logical_width: u32,
    logical_height: u32,
    drawer: Arc<Drawer>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut event_loop_builder = EventLoop::builder();
        let event_loop = EventLoopBuilderExtX11::with_any_thread(&mut event_loop_builder, true)
            .build()
            .unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        let mut window = WinitPixelsWindow::new(logical_width, logical_height, drawer);
        let _ = event_loop.run_app(&mut window);
    })
}

pub struct WinitPixelsWindow {
    logical_width: u32,
    logical_height: u32,
    window: Option<Window>,
    drawer: Arc<Drawer>,
}

impl WinitPixelsWindow {
    pub fn new(logical_width: u32, logical_height: u32, drawer: Arc<Drawer>) -> Self {
        Self {
            logical_width,
            logical_height,
            window: None,
            drawer,
        }
    }
}

impl ApplicationHandler for WinitPixelsWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let size = LogicalSize::new(
                self.logical_width as f64 * 2.0,
                self.logical_height as f64 * 2.0,
            );
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Pixel Display")
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                )
                .unwrap()
        };
        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(self.logical_width, self.logical_height, surface_texture).unwrap()
        };
        pixels.set_present_mode(PresentMode::AutoNoVsync);

        self.drawer.set_pixels_surface(pixels);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(_) = self.drawer.redraw() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                if let Err(_err) = self.drawer.set_pixels_surface_size(size.width, size.height) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.drawer.set_pixels_surface_none();
        self.window.take();
    }
}
