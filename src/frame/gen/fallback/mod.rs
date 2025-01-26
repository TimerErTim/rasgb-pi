use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;
use std::sync::Mutex;
use std::time::Duration;

pub struct FallbackFrameGenerator {
    base_generator: Box<dyn FrameGenerator>,
    fallback_generator: Box<dyn FrameGenerator>,
    idle_duration_micros: u128,
    last_frame_instant: Mutex<Option<u128>>,
}

impl FallbackFrameGenerator {
    pub fn new(
        base_generator: impl FrameGenerator + 'static,
        fallback_generator: impl FrameGenerator + 'static,
        idle_duration: Duration,
    ) -> FallbackFrameGenerator {
        FallbackFrameGenerator {
            base_generator: Box::new(base_generator),
            fallback_generator: Box::new(fallback_generator),
            idle_duration_micros: idle_duration.as_micros(),
            last_frame_instant: Mutex::new(None),
        }
    }
}

impl FrameGenerator for FallbackFrameGenerator {
    fn generate(&self, unix_micros: u128) -> Option<Frame> {
        let base_frame = self.base_generator.generate(unix_micros);
        if let Some(base_frame) = base_frame {
            *self.last_frame_instant.lock().unwrap() = Some(unix_micros);
            return Some(base_frame);
        }

        let last_frame_instant = self.last_frame_instant.lock().unwrap();
        if let Some(last_frame_instant) = last_frame_instant.as_ref() {
            if unix_micros - *last_frame_instant < self.idle_duration_micros {
                return None;
            }
        }

        self.fallback_generator.generate(unix_micros)
    }
}
