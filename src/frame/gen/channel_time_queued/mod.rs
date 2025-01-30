use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;
use std::cmp::Ordering;
use std::collections::{BTreeSet, BinaryHeap};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Eq)]
pub struct ChannelTimedFrame {
    channel: i8,
    unix_micros: u128,
    frame: Frame,
}

impl PartialOrd<Self> for ChannelTimedFrame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChannelTimedFrame {
    fn cmp(&self, other: &Self) -> Ordering {
        self.unix_micros.cmp(&other.unix_micros)
    }
}

pub struct ChannelTimeQueuedFrameGenerator {
    frames: Mutex<BTreeSet<ChannelTimedFrame>>,
    last_frame_meta: Mutex<Option<(i8, u128)>>,
    buffer_size: usize,
    idle_seconds: f64,
}

impl ChannelTimeQueuedFrameGenerator {
    pub fn new(buffer_size: usize, idle_seconds: f64) -> Self {
        let buffer = BTreeSet::new();

        Self {
            frames: Mutex::new(buffer),
            last_frame_meta: Mutex::new(None),
            buffer_size,
            idle_seconds,
        }
    }

    pub fn add_frame(&self, channel: i8, unix_micros: u128, frame: Frame) {
        let mut frames_lock = self.frames.lock().unwrap();
        while frames_lock.len() >= self.buffer_size {
            frames_lock.pop_last();
        }

        let candidate = ChannelTimedFrame {
            channel,
            unix_micros,
            frame,
        };
        if let Some(collision) = frames_lock.get(&candidate) {
            if collision.channel > channel {
                return;
            }
        }
        frames_lock.replace(candidate);
    }

    pub fn is_frame_superseded(&self, channel: i8, unix_micros: u128) -> bool {
        let frames_lock = self.frames.lock().unwrap();
        for frame in frames_lock.iter() {
            if frame.unix_micros > unix_micros {
                break;
            }

            if frame.channel > channel
                && frame.unix_micros + (self.idle_seconds * 1_000_000.0) as u128 > unix_micros
            {
                return true;
            }
        }
        false
    }
}

impl FrameGenerator for ChannelTimeQueuedFrameGenerator {
    fn generate(&self, unix_micros: u128) -> Option<Frame> {
        let mut frames_lock = self.frames.lock().unwrap();
        let mut last_frame_meta = self.last_frame_meta.lock().unwrap();
        let mut candidate: Option<ChannelTimedFrame> = None;

        while let Some(current) = frames_lock.pop_first() {
            if current.unix_micros > unix_micros {
                frames_lock.insert(current);
                break;
            }

            if let Some((last_channel, last_micros)) = last_frame_meta.deref() {
                if *last_channel > current.channel
                    && last_micros + (self.idle_seconds * 1_000_000.0) as u128 > current.unix_micros
                {
                    continue;
                }
            }

            if let Some(candidate) = &candidate {
                if candidate.channel > current.channel
                    && candidate.unix_micros + (self.idle_seconds * 1_000_000.0) as u128
                        > current.unix_micros
                {
                    continue;
                }
            }

            candidate = Some(current);
        }

        if let Some(meta) = &candidate {
            *last_frame_meta = Some((meta.channel, meta.unix_micros));
        }
        candidate.map(|x| x.frame)
    }
}

impl FrameGenerator for Arc<ChannelTimeQueuedFrameGenerator> {
    fn generate(&self, unix_micros: u128) -> Option<Frame> {
        ChannelTimeQueuedFrameGenerator::generate(self, unix_micros)
    }
}
