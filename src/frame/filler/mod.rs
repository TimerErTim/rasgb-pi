pub mod letterboxing;

use crate::display::{Display, DisplayError};
use crate::frame::Frame;

pub trait FrameFiller {
    fn push_to_display(&self, frame: Frame, display: &impl Display) -> Result<(), DisplayError>;
}
