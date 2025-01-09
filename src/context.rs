use crate::config::RasGBConfig;
use crate::display::pixels::PixelsDisplay;
use crate::display::{Display, Pixel};
use crate::frame::filler::letterboxing::LetterboxingDisplayFiller;
use crate::frame::filler::FrameFiller;
use crate::frame::gen::fallback::FallbackFrameGenerator;
use crate::frame::gen::solid_color::SolidColorFrameGenerator;
use crate::frame::gen::time_queried::TimeQueuedFrameGenerator;
use crate::frame::gen::web::{WebQueriedFrameGenerator, WebQueriedFrameGeneratorConfig};
use crate::frame::gen::FrameGenerator;
use crate::web::WebServerConfig;
use std::sync::Arc;
use std::time::Duration;

pub struct RasGBContext {
    pub config: RasGBConfig,

    pub display: Box<dyn Display + 'static>,
    pub generator: Box<dyn FrameGenerator>,
    pub filler: LetterboxingDisplayFiller,
}

impl RasGBContext {
    pub fn testing() -> Self {
        let display = PixelsDisplay::new(192, 128);
        let dimensions = display.dimensions();
        let filler = LetterboxingDisplayFiller::new(Pixel { r: 0, g: 0, b: 0 });
        let web_generator = WebQueriedFrameGenerator::new(WebQueriedFrameGeneratorConfig {
            display_width: dimensions.width,
            display_height: dimensions.height,
            display_fps: 15.0,
        });
        web_generator.start_server(WebServerConfig {
            socket: "0.0.0.0:8081".parse().unwrap(),
            shutdown_signal: None,
        });

        let generator = FallbackFrameGenerator::new(
            web_generator,
            SolidColorFrameGenerator::new(
                Pixel { r: 0, g: 0, b: 0 },
                dimensions.width,
                dimensions.height,
            ),
            Duration::from_secs_f64(5.0),
        );

        Self {
            config: RasGBConfig { fps: 15.0 },
            generator: Box::new(generator),
            display: Box::new(display),
            filler: filler,
        }
    }
}
