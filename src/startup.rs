use crate::config::{DisplayConfigDriver, RasGBConfig};
use crate::context::RasGBContext;
use crate::display::fake::FakeDisplay;
use crate::display::{Display, Pixel};
use crate::frame::filler::letterboxing::LetterboxingDisplayFiller;
use crate::frame::gen::fallback::FallbackFrameGenerator;
use crate::frame::gen::solid_color::SolidColorFrameGenerator;
use crate::frame::gen::web::{WebQueriedFrameGenerator, WebQueriedFrameGeneratorConfig};
use crate::web::WebServerConfig;
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn startup(config: RasGBConfig) -> RasGBContext {
    let mut shutdown_token = CancellationToken::new();

    let display: Box<dyn Display> = config.display.driver.to_display(&config);
    let dimensions = display.dimensions();
    let mut web_generator = WebQueriedFrameGenerator::new(WebQueriedFrameGeneratorConfig {
        channel_idle_seconds: config.timing.idle_seconds.unwrap_or(1.0),
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
        match self.clone() {
            DisplayConfigDriver::WinitPixels { width, height } => {
                #[cfg(not(feature = "winit"))]
                {
                    unimplemented!(
                        "feature `winit` is not enabled but required for display `winit_pixels`"
                    )
                }
                #[cfg(feature = "winit")]
                {
                    use crate::display::pixels::PixelsDisplay;
                    Box::new(PixelsDisplay::new(width, height))
                }
            }
            DisplayConfigDriver::Fake { width, height } => {
                Box::new(FakeDisplay::new(width, height))
            }
            DisplayConfigDriver::Tui { width, height } => {
                #[cfg(not(feature = "tui"))]
                {
                    unimplemented!("feature `tui` is not enabled but required for display `tui`")
                }
                #[cfg(feature = "tui")]
                {
                    use crate::display::tui::TuiDisplay;
                    Box::new(TuiDisplay::new(width, height))
                }
            }
            DisplayConfigDriver::RgbLedMatrix {
                panel_columns,
                panel_rows,
                daisy_chains,
                parallel_chains,
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
                show_refresh_rate,
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

                    Box::new(RgbLedMatrixDisplay::from_options_gen(move || {
                        let mut matrix_options = LedMatrixOptions::default();
                        let mut runtime_options = LedRuntimeOptions::default();

                        runtime_options.set_daemon(false);
                        runtime_options.set_drop_privileges(false);
                        matrix_options.set_refresh_rate(false);

                        matrix_options.set_cols(panel_columns);
                        matrix_options.set_rows(panel_rows);
                        if let Some(daisy_chains) = daisy_chains {
                            matrix_options.set_chain_length(daisy_chains);
                        }
                        if let Some(parallel_chains) = parallel_chains {
                            matrix_options.set_parallel(parallel_chains);
                        }
                        if let Some(pixel_mapper_config) = pixel_mapper_config {
                            matrix_options.set_pixel_mapper_config(pixel_mapper_config.as_str());
                        }

                        if let Some(row_addr_type) = row_addr_type {
                            matrix_options.set_row_addr_type(row_addr_type);
                        }
                        if let Some(led_rgb_sequence) = led_rgb_sequence {
                            matrix_options.set_led_rgb_sequence(led_rgb_sequence.as_str());
                        }
                        if let Some(multiplexing) = multiplexing {
                            matrix_options.set_multiplexing(multiplexing);
                        }
                        if let Some(panel_type) = panel_type {
                            matrix_options.set_panel_type(panel_type.as_str());
                        }

                        if let Some(scan_mode) = scan_mode {
                            matrix_options.set_scan_mode(scan_mode);
                        }
                        if let Some(hardware_pulsing) = hardware_pulsing {
                            matrix_options.set_hardware_pulsing(hardware_pulsing);
                        }
                        if let Some(limit_refresh) = limit_refresh {
                            matrix_options.set_limit_refresh(limit_refresh);
                        }
                        if let Some(pwm_bits) = pwm_bits {
                            matrix_options.set_pwm_bits(pwm_bits).unwrap();
                        }
                        if let Some(pwm_dither_bits) = pwm_dither_bits {
                            matrix_options.set_pwm_dither_bits(pwm_dither_bits);
                        }
                        if let Some(pwm_lsb_nanoseconds) = pwm_lsb_nanoseconds {
                            matrix_options.set_pwm_lsb_nanoseconds(pwm_lsb_nanoseconds);
                        }

                        if let Some(gpio_slowdown) = gpio_slowdown {
                            runtime_options.set_gpio_slowdown(gpio_slowdown);
                        }
                        if let Some(refresh_rate) = show_refresh_rate {
                            matrix_options.set_refresh_rate(refresh_rate);
                        }

                        (Some(matrix_options), Some(runtime_options))
                    }))
                }
            }
        }
    }
}
