pub mod fallback;
pub mod solid_color;
pub mod time_queried;
pub mod web;

use crate::display::Pixel;
use crate::frame::Frame;

pub trait FrameGenerator {
    fn generate(&self, unix_micros: u128) -> Option<Frame>;
}

impl<T: FrameGenerator + ?Sized> FrameGenerator for Box<T> {
    fn generate(&self, unix_micros: u128) -> Option<Frame> {
        (**self).generate(unix_micros)
    }
}
