use crate::display::{Dimensions, Display, DisplayError, Pixel};
use std::cell::RefCell;

pub struct FakeDisplay {
    width: u32,
    height: u32,
    data: RefCell<Vec<Pixel>>,
}

impl FakeDisplay {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: RefCell::new(Vec::new()),
        }
    }
}

impl Display for FakeDisplay {
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            width: self.width,
            height: self.height,
        }
    }

    fn update_pixels(&self, pixels: Vec<Pixel>) -> Result<(), DisplayError> {
        self.data.replace(pixels);
        Ok(())
    }
}
