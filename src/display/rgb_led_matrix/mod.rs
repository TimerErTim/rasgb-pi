use crate::display::{Dimensions, Display, DisplayError, Pixel};
use rpi_led_matrix::{LedColor, LedMatrix, LedMatrixOptions, LedRuntimeOptions};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct RgbLedMatrixDisplay {
    dimensions: Dimensions,
    draw_thread_handle: Option<JoinHandle<()>>,
    data_sender: std::sync::mpsc::Sender<Vec<Pixel>>,
    drop_token: CancellationToken,
}

impl RgbLedMatrixDisplay {
    pub fn new(
        matrix_options: Option<LedMatrixOptions>,
        runtime_options: Option<LedRuntimeOptions>,
    ) -> Self {
        let drop_token = CancellationToken::new();

        let matrix = Arc::new(LedMatrix::new(matrix_options, runtime_options).unwrap());
        let thread_matrix = Arc::clone(&matrix);
        let (width, height) = matrix.canvas().canvas_size();

        let thread_drop_token = drop_token.clone();
        let (data_sender, data_receiver) = std::sync::mpsc::channel::<Vec<Pixel>>();
        let draw_thread_handle = std::thread::spawn(move || {
            while !thread_drop_token.is_cancelled() {
                match data_receiver.recv_timeout(Duration::from_millis(250)) {
                    Ok(pixels) => {
                        let mut canvas = thread_matrix.canvas();
                        for (i, pixel) in pixels.iter().enumerate() {
                            let x = i % width as usize;
                            let y = i / width as usize;
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
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Self {
            dimensions: Dimensions {
                width: width as u32,
                height: height as u32,
            },
            drop_token,
            draw_thread_handle: Some(draw_thread_handle),
            data_sender,
        }
    }
}

impl Display for RgbLedMatrixDisplay {
    fn dimensions(&self) -> Dimensions {
        self.dimensions.clone()
    }

    fn update_pixels(&self, pixels: Vec<Pixel>) -> Result<(), DisplayError> {
        let dimensions = self.dimensions();
        if pixels.len() != dimensions.width as usize * dimensions.height as usize {
            return Err(DisplayError::DimensionMismatch);
        };

        self.data_sender.send(pixels);
        Ok(())
    }
}

impl Drop for RgbLedMatrixDisplay {
    fn drop(&mut self) {
        self.drop_token.cancel();
        let _ = self.draw_thread_handle.take().unwrap().join();
    }
}
