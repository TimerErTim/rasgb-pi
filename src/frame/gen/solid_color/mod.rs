use crate::display::Pixel;
use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;

pub struct SolidColorFrameGenerator {
    color: Pixel,
    width: u32,
    height: u32,
    frame: Frame
}

impl SolidColorFrameGenerator {
    pub fn new(color: Pixel, width: u32, height: u32) -> Self {
        Self {
            color: color.clone(),
            width,
            height,
            frame: Frame::with_color(width, height, color),
        }
    }
}

impl FrameGenerator for SolidColorFrameGenerator {
    fn generate(&self, _unix_micros: u128) -> Option<Frame> {
        Some(self.frame.clone())
    }
}
