use crate::display::pixels::drawer::Drawer;
use pixels::wgpu::PresentMode;
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{Window, WindowId};

pub fn create_pixels_window(
    logical_width: u32,
    logical_height: u32,
    drawer: Arc<Drawer>,
    shutdown_token: CancellationToken,
) -> impl FnOnce() -> () {
    let mut application = WinitPixelsWindow::new(
        logical_width,
        logical_height,
        drawer,
        shutdown_token.clone(),
    );
    let window = Arc::clone(&application.window);
    let window_thread_handle = std::thread::spawn(move || {
        let mut event_loop_builder = EventLoop::builder();
        let event_loop = EventLoopBuilderExtX11::with_any_thread(&mut event_loop_builder, true)
            .build()
            .unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        let _ = event_loop.run_app(&mut application);
    });
    move || {
        shutdown_token.cancel();
        let window_guard = window.lock().unwrap();
        if let Some(window) = window_guard.as_ref() {
            window.request_redraw();
        }
        drop(window_guard);
        window_thread_handle.join().unwrap();
    }
}

pub struct WinitPixelsWindow {
    logical_width: u32,
    logical_height: u32,
    window: Arc<Mutex<Option<Window>>>,
    drawer: Arc<Drawer>,
    shutdown_token: CancellationToken,
}

impl WinitPixelsWindow {
    pub fn new(
        logical_width: u32,
        logical_height: u32,
        drawer: Arc<Drawer>,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            logical_width,
            logical_height,
            window: Arc::new(Mutex::new(None)),
            drawer,
            shutdown_token,
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
        let mut window_guard = self.window.lock().unwrap();
        *window_guard = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.shutdown_token.is_cancelled() {
            event_loop.exit();
            return;
        }

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
        let window_guard = self.window.lock().unwrap();
        if let Some(window) = window_guard.as_ref() {
            window.request_redraw();
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.drawer.set_pixels_surface_none();
        let mut window_guard = self.window.lock().unwrap();
        window_guard.take();
    }
}
