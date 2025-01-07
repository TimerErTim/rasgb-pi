pub mod fallback;
pub mod solid_color;
pub mod time_queried;

use crate::display::Pixel;
use crate::frame::Frame;

pub trait FrameGenerator {
    fn generate(&self, unix_micros: u128) -> Option<Frame>;
}
