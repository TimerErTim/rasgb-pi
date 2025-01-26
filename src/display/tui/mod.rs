mod drawer;

use crate::display::tui::drawer::Drawer;
use crate::display::{Dimensions, Display, DisplayError, Pixel};
use image::{DynamicImage, GenericImageView};
use tokio_util::sync::CancellationToken;

pub struct TuiDisplay {
    width: u32,
    height: u32,
    drawer: Drawer,
    drop_token: CancellationToken,
}

impl TuiDisplay {
    pub fn new(width: u32, height: u32) -> Self {
        let config = viuer::Config {
            restore_cursor: false,
            ..Default::default()
        };
        let drop_token = CancellationToken::new();

        let drawer = Drawer::new(config, drop_token.clone());

        Self {
            width,
            height,
            drop_token,
            drawer,
        }
    }
}

impl Display for TuiDisplay {
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

        let frame_buffer = image::RgbImage::from_vec(
            self.width,
            self.height,
            pixels
                .into_iter()
                .flat_map(|pixel| [pixel.r, pixel.g, pixel.b])
                .collect(),
        )
        .unwrap();
        let image = DynamicImage::ImageRgb8(frame_buffer);
        self.drawer.send_image(image);

        Ok(())
    }
}

impl Drop for TuiDisplay {
    fn drop(&mut self) {
        self.drop_token.cancel();
        let _ = self.drawer.thread_handle.take().unwrap().join();
    }
}
