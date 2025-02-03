use crate::config::read_config_from_env;
use crate::display::Display;
use crate::frame::filler::FrameFiller;
use crate::run::run;
use crate::shutdown::shutdown;
use crate::startup::startup;

mod config;
mod context;
mod display;
mod frame;
mod run;
mod shutdown;
mod startup;
mod web;
mod lib;

#[tokio::main]
async fn main() {
    let config = read_config_from_env().unwrap();
    let context = startup(config).await;
    let _ = run(&context).await;
    shutdown(context).await;
}
