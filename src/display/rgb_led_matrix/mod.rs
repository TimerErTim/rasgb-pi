use crate::display::{Dimensions, Display, DisplayError, Pixel};
use crate::lib::BlockingOption;
use rpi_led_matrix::{LedColor, LedMatrix, LedMatrixOptions, LedRuntimeOptions};
use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct RgbLedMatrixDisplay {
    dimensions: Dimensions,
    draw_thread_handle: Option<JoinHandle<()>>,
    data_sender: BlockingOption<Vec<Pixel>>,
    drop_token: CancellationToken,
}

impl RgbLedMatrixDisplay {
    pub fn from_options_gen(
        options: impl FnOnce() -> (Option<LedMatrixOptions>, Option<LedRuntimeOptions>) + Send + 'static,
    ) -> Self {
        let drop_token = CancellationToken::new();

        let thread_drop_token = drop_token.clone();
        let (dimension_sender, dimension_receiver) = channel::<Dimensions>();

        let pixels_receiver = BlockingOption::<Vec<Pixel>>::new();
        let pixels_sender = pixels_receiver.clone();
        let draw_thread_handle = std::thread::spawn(move || {
            let (matrix_options, runtime_options) = options();
            let matrix = LedMatrix::new(matrix_options, runtime_options).unwrap();
            let mut canvas = matrix.offscreen_canvas();
            let (width, height) = canvas.canvas_size();
            dimension_sender
                .send(Dimensions {
                    width: width as u32,
                    height: height as u32,
                })
                .unwrap();
            drop(dimension_sender);

            while !thread_drop_token.is_cancelled() {
                match pixels_receiver.recv_timeout(Duration::from_millis(250)) {
                    Some(pixels) => {
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
                        canvas = matrix.swap(canvas);
                    }
                    _ => {}
                }
            }
        });

        Self {
            dimensions: dimension_receiver.recv().unwrap(),
            drop_token,
            draw_thread_handle: Some(draw_thread_handle),
            data_sender: pixels_sender,
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
