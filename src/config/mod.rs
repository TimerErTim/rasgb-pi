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
    WinitPixels { width: u32, height: u32 },
    #[serde(rename = "fake")]
    Fake { width: u32, height: u32 },
    #[serde(rename = "ratatui")]
    Ratatui,
    #[serde(rename = "rgb_led_matrix")]
    RgbLedMatrix {
        panel_rows: u32,
        panel_columns: u32,
        daisy_chains: Option<u32>,
        parallel_chains: Option<u32>,
        pixel_mapper_config: Option<String>,

        row_addr_type: Option<u32>,
        led_rgb_sequence: Option<String>,
        multiplexing: Option<u32>,
        panel_type: Option<String>,

        scan_mode: Option<u32>,
        hardware_pulsing: Option<bool>,
        limit_refresh: Option<u32>,
        pwm_bits: Option<u8>,
        pwm_dither_bits: Option<u32>,
        pwm_lsb_nanoseconds: Option<u32>,

        gpio_slowdown: Option<u32>,
    },
}
