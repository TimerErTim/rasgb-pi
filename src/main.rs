use crate::display::fake::FakeDisplay;
use crate::display::pixels::PixelsDisplay;
use crate::display::{Display, Pixel};
use crate::frame::filler::letterboxing::LetterboxingDisplayFiller;
use crate::frame::filler::FrameFiller;
use crate::frame::gen::fallback::FallbackFrameGenerator;
use crate::frame::gen::solid_color::SolidColorFrameGenerator;
use crate::frame::gen::time_queried::TimeQueuedFrameGenerator;
use crate::frame::Frame;
use crate::sync::sync_pixels;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

mod display;
mod frame;
mod sync;

#[tokio::main]
async fn main() {
    let display = PixelsDisplay::new(192, 128);
    let mut dimensions = display.dimensions();

    // Fake smaller frame
    dimensions.width -= 64;
    dimensions.height -= 64;

    let fps = 60;
    let sps = 500;

    let filler = LetterboxingDisplayFiller::new(Pixel { r: 0, g: 255, b: 0 });
    let queued_generator = TimeQueuedFrameGenerator::new(100_000);
    let queued_generator = Arc::new(queued_generator);
    let thread_generator = Arc::clone(&queued_generator);

    let generator = FallbackFrameGenerator::new(
        queued_generator,
        SolidColorFrameGenerator::new(
            Pixel {
                r: 0,
                g: 255,
                b: 255,
            },
            dimensions.width,
            dimensions.height,
        ),
        Duration::from_secs_f64(0.1),
    );

    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let unix_duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let pixel_count = dimensions.width * dimensions.height;
        for pixel in 0..pixel_count {
            let mut pixels = Vec::with_capacity(pixel_count as usize);
            for i in 0..pixel_count {
                if i < pixel {
                    pixels.push(Pixel { r: 255, g: 0, b: 0 });
                } else {
                    pixels.push(Pixel { r: 0, g: 0, b: 255 });
                }
            }

            let frame = Frame::new(dimensions.width, dimensions.height, pixels).unwrap();
            let current_unix_micros = unix_duration.as_micros()
                + ((pixel as f64 / sps as f64 + 3.0) * 1_000_000.0) as u128;

            thread_generator.add_frame(current_unix_micros, frame);

            std::thread::sleep(Duration::from_secs_f64(1.0 / sps as f64));
        }
        let current_unix_micros = unix_duration.as_micros()
            + ((pixel_count as f64 / sps as f64 + 4.0) * 1_000_000.0) as u128;
        let frame = Frame::new(
            1,
            1,
            vec![Pixel {
                r: 255,
                g: 0,
                b: 255,
            }],
        )
        .unwrap();
        thread_generator.add_frame(current_unix_micros, frame);
    });

    sync_pixels(&display, &filler, &generator, fps).await;
}
