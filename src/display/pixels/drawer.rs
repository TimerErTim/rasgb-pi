use crate::display::Pixel;
use pixels::{Pixels, TextureError};
use std::sync::Mutex;

pub struct Drawer {
    pixels_surface: Mutex<Option<Pixels>>,
}

impl Drawer {
    pub fn new() -> Self {
        Self {
            pixels_surface: Mutex::new(None),
        }
    }

    pub fn set_data(&self, data: Vec<Pixel>) {
        let mut surface_lock = self.pixels_surface.lock().unwrap();
        if let Some(pixels) = surface_lock.as_mut() {
            let buffer_iterator = pixels.frame_mut().chunks_exact_mut(4);
            let data_iterator = data.iter().map(|pixel| [pixel.r, pixel.g, pixel.b, 255]);
            for (buffer_slice, data_slice) in buffer_iterator.zip(data_iterator) {
                buffer_slice.copy_from_slice(&data_slice);
            }
        }
    }

    pub fn set_pixels_surface(&self, pixels: Pixels) {
        let mut surface_lock = self.pixels_surface.lock().unwrap();
        *surface_lock = Some(pixels);
    }

    pub fn set_pixels_surface_none(&self) {
        let mut surface_lock = self.pixels_surface.lock().unwrap();
        *surface_lock = None;
    }

    pub fn set_pixels_surface_size(&self, width: u32, height: u32) -> Result<(), TextureError> {
        let mut surface_lock = self.pixels_surface.lock().unwrap();
        if let Some(pixels) = surface_lock.as_mut() {
            pixels.resize_surface(width, height)?;
        }
        Ok(())
    }

    pub fn redraw(&self) -> Result<(), pixels::Error> {
        let surface_lock = self.pixels_surface.lock().unwrap();
        if let Some(pixels) = surface_lock.as_ref() {
            pixels.render()?;
        }
        Ok(())
    }
}
