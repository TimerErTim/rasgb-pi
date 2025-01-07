use crate::display::{Dimensions, Pixel};
use thiserror::Error;

pub mod filler;
pub mod gen;

#[derive(Clone, PartialEq, Eq)]
pub struct Frame {
    width: u32,
    height: u32,
    pixel_data: Vec<Pixel>,
}

impl Frame {
    pub fn new(width: u32, height: u32, pixel_data: Vec<Pixel>) -> Result<Self, FrameError> {
        if pixel_data.len() != (width * height) as usize {
            return Err(FrameError::DimensionMismatch);
        }

        Ok(Self {
            width,
            height,
            pixel_data,
        })
    }

    pub fn dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.width,
            height: self.height,
        }
    }

    pub fn pixel_data(&self) -> &Vec<Pixel> {
        &self.pixel_data
    }
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("the amount of provided pixels does not correspond to the dimensions of the frame")]
    DimensionMismatch,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
