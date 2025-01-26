use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Eq)]
pub struct TimedFrame {
    unix_micros: u128,
    frame: Frame,
}

impl PartialOrd<Self> for TimedFrame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimedFrame {
    fn cmp(&self, other: &Self) -> Ordering {
        other.unix_micros.cmp(&self.unix_micros)
    }
}

pub struct TimeQueuedFrameGenerator {
    frames: Mutex<BinaryHeap<TimedFrame>>,
}

impl TimeQueuedFrameGenerator {
    pub fn new(buffer_size: usize) -> Self {
        let buffer = BinaryHeap::with_capacity(buffer_size);

        Self {
            frames: Mutex::new(buffer),
        }
    }

    pub fn add_frame(&self, unix_micros: u128, frame: Frame) {
        let mut frames_lock = self.frames.lock().unwrap();
        while frames_lock.len() >= frames_lock.capacity() {
            frames_lock.pop();
        }
        frames_lock.push(TimedFrame { unix_micros, frame });
    }
}

impl FrameGenerator for TimeQueuedFrameGenerator {
    fn generate(&self, unix_micros: u128) -> Option<Frame> {
        let mut frames_lock = self.frames.lock().unwrap();
        let mut prev_frame = None;

        while let Some(frame) = frames_lock.peek() {
            if frame.unix_micros > unix_micros {
                break;
            }
            let current = frames_lock.pop().unwrap();
            prev_frame = Some(current.frame);
        }

        prev_frame
    }
}

impl FrameGenerator for Arc<TimeQueuedFrameGenerator> {
    fn generate(&self, unix_micros: u128) -> Option<Frame> {
        TimeQueuedFrameGenerator::generate(self, unix_micros)
    }
}
