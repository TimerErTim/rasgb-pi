use serde::{Deserialize, Serialize};
use std::net::IpAddr;

mod load;

pub use load::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct RasGBConfig {
    pub display: DisplayConfig,
    pub server: ServerConfig,
    pub timing: TimingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub driver: DisplayConfigDriver,
}

fn default_ip() -> IpAddr {
    IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
}
fn default_port() -> u16 {
    8081
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_ip")]
    pub ip: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimingConfig {
    pub idle_seconds: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayConfigDriver {
    #[serde(rename = "winit_pixels")]
    WinitPixels,
    #[serde(rename = "fake")]
    Fake,
    #[serde(rename = "ratatui")]
    Ratatui,
    #[serde(rename = "rgb_led_matrix")]
    RgbLedMatrix { idk_yet: String },
}
