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

    let display: Box<dyn Display> = config.display.driver.to_display(&config);
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

impl DisplayConfigDriver {
    pub fn to_display(&self, config: &RasGBConfig) -> Box<dyn Display> {
        match self {
            DisplayConfigDriver::WinitPixels { width, height } => {
                Box::new(PixelsDisplay::new(*width, *height))
            }
            DisplayConfigDriver::Fake { width, height } => {
                Box::new(FakeDisplay::new(*width, *height))
            }
            DisplayConfigDriver::Ratatui => {
                unimplemented!()
            }
            DisplayConfigDriver::RgbLedMatrix {
                panel_columns,
                panel_rows,
                daisy_chains,
                parallel_channels,
                pixel_mapper_config,

                row_addr_type,
                led_rgb_sequence,
                multiplexing,
                panel_type,
                scan_mode,
                hardware_pulsing,
                limit_refresh,
                pwm_bits,
                pwm_dither_bits,
                pwm_lsb_nanoseconds,

                gpio_slowdown,
            } => {
                #[cfg(not(feature = "rpi"))]
                {
                    dbg!(config);
                    unimplemented!(
                        "feature `rpi` is not enabled but required for display `rgb_led_matrix`"
                    )
                }
                #[cfg(feature = "rpi")]
                {
                    use crate::display::rgb_led_matrix::RgbLedMatrixDisplay;
                    use rpi_led_matrix::{LedMatrixOptions, LedRuntimeOptions};

                    let mut matrix_options = LedMatrixOptions::default();
                    let mut runtime_options = LedRuntimeOptions::default();

                    matrix_options.set_cols(*panel_columns);
                    matrix_options.set_rows(*panel_rows);
                    if let Some(daisy_chains) = daisy_chains {
                        matrix_options.set_chain_length(*daisy_chains);
                    }
                    if let Some(parallel_channels) = parallel_channels {
                        matrix_options.set_parallel(*parallel_channels);
                    }
                    if let Some(pixel_mapper_config) = pixel_mapper_config {
                        matrix_options.set_pixel_mapper_config(pixel_mapper_config.as_str());
                    }

                    if let Some(row_addr_type) = row_addr_type {
                        matrix_options.set_row_addr_type(*row_addr_type);
                    }
                    if let Some(led_rgb_sequence) = led_rgb_sequence {
                        matrix_options.set_led_rgb_sequence(led_rgb_sequence.as_str());
                    }
                    if let Some(multiplexing) = multiplexing {
                        matrix_options.set_multiplexing(*multiplexing);
                    }
                    if let Some(panel_type) = panel_type {
                        matrix_options.set_panel_type(panel_type.as_str());
                    }

                    if let Some(scan_mode) = scan_mode {
                        matrix_options.set_scan_mode(*scan_mode);
                    }
                    if let Some(hardware_pulsing) = hardware_pulsing {
                        matrix_options.set_hardware_pulsing(*hardware_pulsing);
                    }
                    if let Some(limit_refresh) = limit_refresh {
                        matrix_options.set_limit_refresh(*limit_refresh);
                    }
                    if let Some(pwm_bits) = pwm_bits {
                        matrix_options.set_pwm_bits(*pwm_bits).unwrap();
                    }
                    if let Some(pwm_dither_bits) = pwm_dither_bits {
                        matrix_options.set_pwm_dither_bits(*pwm_dither_bits);
                    }
                    if let Some(pwm_lsb_nanoseconds) = pwm_lsb_nanoseconds {
                        matrix_options.set_pwm_lsb_nanoseconds(*pwm_lsb_nanoseconds);
                    }

                    if let Some(gpio_slowdown) = gpio_slowdown {
                        runtime_options.set_gpio_slowdown(*gpio_slowdown);
                    }

                    dbg!(&matrix_options, &runtime_options);

                    Box::new(RgbLedMatrixDisplay::new(
                        Some(matrix_options),
                        Some(runtime_options),
                    ))
                }
            }
        }
    }
}
