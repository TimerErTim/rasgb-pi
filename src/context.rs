use crate::config::RasGBConfig;
use crate::display::pixels::PixelsDisplay;
use crate::display::{Display, Pixel};
use crate::frame::filler::letterboxing::LetterboxingDisplayFiller;
use crate::frame::filler::FrameFiller;
use crate::frame::gen::fallback::FallbackFrameGenerator;
use crate::frame::gen::solid_color::SolidColorFrameGenerator;
use crate::frame::gen::time_queued::TimeQueuedFrameGenerator;
use crate::frame::gen::web::{WebQueriedFrameGenerator, WebQueriedFrameGeneratorConfig};
use crate::frame::gen::FrameGenerator;
use crate::web::WebServerConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct RasGBContext {
    pub config: RasGBConfig,

    pub display: Box<dyn Display + 'static>,
    pub generator: Box<dyn FrameGenerator>,
    pub filler: LetterboxingDisplayFiller,

    pub shutdown_token: CancellationToken,
}
