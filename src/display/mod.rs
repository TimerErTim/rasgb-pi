pub mod fake;
pub mod pixels;
#[cfg(feature = "rpi")]
pub mod rgb_led_matrix;

use crate::frame::gen::FrameGenerator;
use crate::frame::Frame;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub trait Display {
    fn dimensions(&self) -> Dimensions;

    fn update_pixels(&self, pixels: Vec<Pixel>) -> Result<(), DisplayError>;
}

impl<T: Display + ?Sized> Display for Box<T> {
    fn dimensions(&self) -> Dimensions {
        (**self).dimensions()
    }

    fn update_pixels(&self, pixels: Vec<Pixel>) -> Result<(), DisplayError> {
        (**self).update_pixels(pixels)
    }
}

#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("the amount of provided pixels does not correspond to the dimensions of the display")]
    DimensionMismatch,
    #[error("the provided frame exceeds the dimensions of the display")]
    FrameTooLarge,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
