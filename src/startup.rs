use crate::config::{DisplayConfigDriver, RasGBConfig};
use crate::context::RasGBContext;
use crate::display::fake::FakeDisplay;
use crate::display::pixels::PixelsDisplay;
use crate::display::{Display, Pixel};
use crate::frame::filler::letterboxing::LetterboxingDisplayFiller;
use crate::frame::gen::fallback::FallbackFrameGenerator;
use crate::frame::gen::solid_color::SolidColorFrameGenerator;
use crate::frame::gen::web::{WebQueriedFrameGenerator, WebQueriedFrameGeneratorConfig};
use crate::web::WebServerConfig;
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn startup(config: RasGBConfig) -> RasGBContext {
    let mut shutdown_token = CancellationToken::new();

    let display: Box<dyn Display> = match &config.display.driver {
        DisplayConfigDriver::WinitPixels => Box::new(PixelsDisplay::new(
            config.display.width,
            config.display.height,
        )),
        DisplayConfigDriver::Fake => Box::new(FakeDisplay::new(
            config.display.width,
            config.display.height,
        )),
        DisplayConfigDriver::Ratatui => {
            unimplemented!()
        }
        DisplayConfigDriver::RgbLedMatrix { .. } => {
            todo!()
        }
    };
    let dimensions = display.dimensions();
    let mut web_generator = WebQueriedFrameGenerator::new(WebQueriedFrameGeneratorConfig {
        display_width: dimensions.width,
        display_height: dimensions.height,
        display_fps: config.display.fps,
    });

    let server_shutdown = shutdown_token.clone();
    web_generator.start_server(WebServerConfig {
        socket: SocketAddr::new(config.server.ip, config.server.port),
        shutdown_signal: Some(Box::pin(async move {
            server_shutdown.cancelled().await;
        })),
    });

    let generator = FallbackFrameGenerator::new(
        web_generator,
        SolidColorFrameGenerator::new(
            Pixel { r: 0, g: 0, b: 0 },
            dimensions.width,
            dimensions.height,
        ),
        Duration::from_secs_f64(f64::max(
            config.timing.idle_seconds.unwrap_or(1.0),
            1.0 / config.display.fps,
        )),
    );

    RasGBContext {
        config,
        generator: Box::new(generator),
        display,
        filler: LetterboxingDisplayFiller::new(Pixel { r: 0, g: 0, b: 0 }),
        shutdown_token,
    }
}
