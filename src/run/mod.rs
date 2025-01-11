mod signals;
mod sync;

use crate::context::RasGBContext;
use crate::run::signals::exit_signal;
use crate::run::sync::sync_frames;
use std::time::Duration;
use tokio::time::{interval, MissedTickBehavior};

pub async fn run(context: &RasGBContext) {
    let config = &context.config;
    let mut interval = interval(Duration::from_secs_f64(1.0 / config.display.fps));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = interval.tick() => {},
            _ = exit_signal() => break
        }

        sync_frames(&context.display, &context.filler, &context.generator);
    }
}
