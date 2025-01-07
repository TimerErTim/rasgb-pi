use crate::display::pixels::drawer::Drawer;
use crate::display::{Dimensions, Display, DisplayError, Pixel};
use std::sync::Arc;

mod drawer;
mod winit;

pub struct PixelsDisplay {
    width: u32,
    height: u32,
    drawer: Arc<Drawer>,
    window_thread_handle: std::thread::JoinHandle<()>,
}

impl PixelsDisplay {
    pub fn new(width: u32, height: u32) -> Self {
        let drawer = Arc::new(Drawer::new());
        let window_thread_handle = winit::create_pixels_window(width, height, Arc::clone(&drawer));

        Self {
            width,
            height,
            drawer,
            window_thread_handle,
        }
    }
}

impl Display for PixelsDisplay {
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.width,
            height: self.height,
        }
    }

    fn update_pixels(&self, pixels: Vec<Pixel>) -> Result<(), DisplayError> {
        if pixels.len() != self.width as usize * self.height as usize {
            return Err(DisplayError::DimensionMismatch);
        }

        self.drawer.set_data(pixels);

        Ok(())
    }
}
