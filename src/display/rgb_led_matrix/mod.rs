use crate::display::{Dimensions, Display, DisplayError, Pixel};
use rpi_led_matrix::{LedColor, LedMatrix, LedMatrixOptions, LedRuntimeOptions};

pub struct RgbLedMatrixDisplay {
    matrix: LedMatrix,
}

impl RgbLedMatrixDisplay {
    pub fn new(
        matrix_options: Option<LedMatrixOptions>,
        runtime_options: Option<LedRuntimeOptions>,
    ) -> Self {
        Self {
            matrix: LedMatrix::new(matrix_options, runtime_options).unwrap(),
        }
    }
}

impl Display for RgbLedMatrixDisplay {
    fn dimensions(&self) -> Dimensions {
        let (width, height) = self.matrix.canvas().canvas_size();
        Dimensions {
            width: width as u32,
            height: height as u32,
        }
    }

    fn update_pixels(&self, pixels: Vec<Pixel>) -> Result<(), DisplayError> {
        let dimensions = self.dimensions();
        if pixels.len() != dimensions.width as usize * dimensions.height as usize {
            return Err(DisplayError::DimensionMismatch);
        };

        let mut canvas = self.matrix.canvas();
        for (i, pixel) in pixels.iter().enumerate() {
            let x = i % dimensions.width as usize;
            let y = i / dimensions.width as usize;
            canvas.set(
                x as i32,
                y as i32,
                &LedColor {
                    red: pixel.r,
                    green: pixel.g,
                    blue: pixel.b,
                },
            );
        }
        Ok(())
    }
}
