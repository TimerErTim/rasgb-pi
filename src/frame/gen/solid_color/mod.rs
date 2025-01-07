use crate::display::Pixel;
use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;

pub struct SolidColorFrameGenerator {
    color: Pixel,
    width: u32,
    height: u32,
}

impl SolidColorFrameGenerator {
    pub fn new(color: Pixel, width: u32, height: u32) -> Self {
        Self {
            color,
            width,
            height,
        }
    }
}

impl FrameGenerator for SolidColorFrameGenerator {
    fn generate(&self, _unix_micros: u128) -> Option<Frame> {
        Some(Frame {
            width: self.width,
            height: self.height,
            pixel_data: vec![self.color.clone(); (self.width * self.height) as usize],
        })
    }
}
