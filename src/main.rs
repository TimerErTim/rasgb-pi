use crate::context::RasGBContext;
use crate::display::fake::FakeDisplay;
use crate::display::pixels::PixelsDisplay;
use crate::display::{Display, Pixel};
use crate::frame::filler::letterboxing::LetterboxingDisplayFiller;
use crate::frame::filler::FrameFiller;
use crate::frame::gen::fallback::FallbackFrameGenerator;
use crate::frame::gen::solid_color::SolidColorFrameGenerator;
use crate::frame::gen::time_queried::TimeQueuedFrameGenerator;
use crate::frame::Frame;
use crate::run::run;
use crate::shutdown::shutdown;
use crate::startup::startup;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

mod config;
mod context;
mod display;
mod frame;
mod run;
mod shutdown;
mod startup;
mod web;

#[tokio::main]
async fn main() {
    let context = startup().await;
    let _ = run(&context).await;
    shutdown(context).await;
}
