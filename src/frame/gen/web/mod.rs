use crate::frame::gen::time_queried::TimeQueuedFrameGenerator;
use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;
use crate::web;
use crate::web::{WebServerConfig, WebServerControl};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::task;

#[derive(Clone)]
pub struct WebQueriedFrameGeneratorConfig {
    pub display_width: u32,
    pub display_height: u32,
    pub display_fps: f64,
}

pub struct WebQueriedFrameGenerator {
    config: WebQueriedFrameGeneratorConfig,
    time_queued_frame_generator: Arc<TimeQueuedFrameGenerator>,
    server_join_handles: Vec<task::JoinHandle<()>>,
}

impl WebQueriedFrameGenerator {
    pub fn new(config: WebQueriedFrameGeneratorConfig) -> Self {
        let generator = TimeQueuedFrameGenerator::new(2_500);

        Self {
            config,
            time_queued_frame_generator: Arc::new(generator),
            server_join_handles: vec![],
        }
    }

    pub fn start_server(&mut self, config: WebServerConfig) {
        let framed_generator = Arc::clone(&self.time_queued_frame_generator);
        let gen_config = self.config.clone();
        let server_control = WebServerControl {
            display_width: self.config.display_width,
            display_height: self.config.display_height,
            display_fps: self.config.display_fps,
            on_frame_received: Box::new(move |unix_micros, frame| {
                if frame.width > gen_config.display_width
                    || frame.height > gen_config.display_height
                {
                    return Err("frame too large".to_string());
                }

                framed_generator.add_frame(unix_micros, frame);
                Ok(())
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
