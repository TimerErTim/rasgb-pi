use crate::display::pixels::drawer::Drawer;
use crate::display::{Dimensions, Display, DisplayError, Pixel};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

mod drawer;
mod winit;

pub struct PixelsDisplay {
    width: u32,
    height: u32,
    drawer: Arc<Drawer>,
    close_window_handle: Option<Box<dyn FnOnce() -> ()>>,
    shutdown_token: CancellationToken,
}

impl PixelsDisplay {
    pub fn new(width: u32, height: u32) -> Self {
        let shutdown_token = CancellationToken::new();
        let drawer = Arc::new(Drawer::new());
        let close_window_handle =
            winit::create_pixels_window(width, height, Arc::clone(&drawer), shutdown_token.clone());

        Self {
            width,
            height,
            drawer,
            close_window_handle: Some(Box::new(close_window_handle)),
            shutdown_token,
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

impl Drop for PixelsDisplay {
    fn drop(&mut self) {
        self.close_window_handle.take().unwrap()();
    }
}
