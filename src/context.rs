use crate::config::RasGBConfig;
use crate::display::Display;
use crate::frame::filler::letterboxing::LetterboxingDisplayFiller;
use crate::frame::gen::FrameGenerator;
use tokio_util::sync::CancellationToken;

pub struct RasGBContext {
    pub config: RasGBConfig,

    pub display: Box<dyn Display + 'static>,
    pub generator: Box<dyn FrameGenerator>,
    pub filler: LetterboxingDisplayFiller,

    pub shutdown_token: CancellationToken,
}
