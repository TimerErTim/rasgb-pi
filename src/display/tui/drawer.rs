use image::DynamicImage;
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct Drawer {
    pub thread_handle: Option<JoinHandle<()>>,
    pixels_sender: Sender<DynamicImage>,
}

impl Drawer {
    pub fn new(config: viuer::Config, drop_token: CancellationToken) -> Self {
        let (pixels_sender, pixels_receiver) = std::sync::mpsc::channel();

        let thread_handle = std::thread::spawn(move || {
            while !drop_token.is_cancelled() {
                match pixels_receiver.recv_timeout(Duration::from_millis(250)) {
                    Ok(ref image) => {
                        let _ = viuer::print(image, &config);
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    _ => {}
                }
            }
        });

        Self {
            thread_handle: Some(thread_handle),
            pixels_sender,
        }
    }

    pub fn send_image(&self, image: DynamicImage) {
        let _ = self.pixels_sender.send(image);
    }
}
