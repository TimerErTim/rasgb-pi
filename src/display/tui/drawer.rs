use crate::display::Pixel;
use image::DynamicImage;
use std::sync::mpsc::Sender;
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
                match pixels_receiver.try_recv() {
                    Ok(ref image) => {
                        let _ = viuer::print(image, &config);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
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
