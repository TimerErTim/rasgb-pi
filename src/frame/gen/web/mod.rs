use crate::frame::gen::channel_time_queued::ChannelTimeQueuedFrameGenerator;
use crate::frame::gen::time_queued::TimeQueuedFrameGenerator;
use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;
use crate::web;
use crate::web::{WebServerConfig, WebServerControl};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::task;

#[derive(Clone)]
pub struct WebQueriedFrameGeneratorConfig {
    pub channel_idle_seconds: f64,
    pub display_width: u32,
    pub display_height: u32,
    pub display_fps: f64,
}

pub struct WebQueriedFrameGenerator {
    config: WebQueriedFrameGeneratorConfig,
    time_queued_frame_generator: Arc<ChannelTimeQueuedFrameGenerator>,
    server_join_handles: Vec<task::JoinHandle<()>>,
}

impl WebQueriedFrameGenerator {
    pub fn new(config: WebQueriedFrameGeneratorConfig) -> Self {
        let generator = ChannelTimeQueuedFrameGenerator::new(2_500, config.channel_idle_seconds);

        Self {
            config,
            time_queued_frame_generator: Arc::new(generator),
            server_join_handles: vec![],
        }
    }

    pub fn start_server(&mut self, config: WebServerConfig) {
        let server_control = WebServerControl {
            display_width: self.config.display_width,
            display_height: self.config.display_height,
            display_fps: self.config.display_fps,
            on_frame_received: Box::new({
                let framed_generator = Arc::clone(&self.time_queued_frame_generator);
                let gen_config = self.config.clone();
                move |event| {
                    if event.frame.width > gen_config.display_width
                        || event.frame.height > gen_config.display_height
                    {
                        return Err("frame too large".to_string());
                    }

                    framed_generator.add_frame(
                        event.channel.unwrap_or(0),
                        event.unix_micros,
                        event.frame,
                    );
                    Ok(())
                }
            }),
            on_frame_superseded_check: Box::new({
                let frame_gen = Arc::clone(&self.time_queued_frame_generator);
                move |event| {
                    frame_gen.is_frame_superseded(event.channel.unwrap_or(0), event.unix_micros)
                }
            }),
        };

        let handle = task::spawn(async move {
            web::run_server(config, server_control).await;
        });
        self.server_join_handles.push(handle);
    }
}

impl FrameGenerator for WebQueriedFrameGenerator {
    fn generate(&self, unix_micros: u128) -> Option<Frame> {
        self.time_queued_frame_generator.generate(unix_micros)
    }
}

impl Drop for WebQueriedFrameGenerator {
    fn drop(&mut self) {
        task::block_in_place(|| {
            let runtime = Handle::current();
            for handle in self.server_join_handles.drain(..) {
                runtime.block_on(async {
                    handle.await;
                })
            }
        })
    }
}
