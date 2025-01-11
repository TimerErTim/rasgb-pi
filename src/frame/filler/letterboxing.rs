use crate::display::{Display, DisplayError, Pixel};
use crate::frame::filler::FrameFiller;
use crate::frame::Frame;
use anyhow::anyhow;

pub struct LetterboxingDisplayFiller {
    background_color: Pixel,
}

impl LetterboxingDisplayFiller {
    pub fn new(background_color: Pixel) -> Self {
        Self { background_color }
    }
}

impl FrameFiller for LetterboxingDisplayFiller {
    fn push_to_display(&self, frame: Frame, display: &dyn Display) -> Result<(), DisplayError> {
        let dimensions = display.dimensions();
        if frame.width > dimensions.width || frame.height > dimensions.height {
            return Err(DisplayError::FrameTooLarge);
        }

        let padding_top = (dimensions.height - frame.height) / 2;
        let padding_bottom = dimensions.height - frame.height - padding_top;
        let padding_left = (dimensions.width - frame.width) / 2;
        let padding_right = dimensions.width - frame.width - padding_left;

        let mut frame_pixels = frame.pixel_data.into_iter();
        let mut pixels = Vec::with_capacity((dimensions.height * dimensions.width) as usize);
        let mut bg_iter = std::iter::repeat(self.background_color.clone());
        // Push top padding
        pixels.extend(
            bg_iter
                .clone()
                .take(padding_top as usize * dimensions.width as usize),
        );
        for _ in 0..frame.height {
            // Push left padding
            pixels.extend(bg_iter.clone().take(padding_left as usize));
            // Push frame
            pixels.extend(frame_pixels.by_ref().take(frame.width as usize));
            // Push right padding
            pixels.extend(bg_iter.clone().take(padding_right as usize));
        }
        // Push bottom padding
        pixels.extend(
            bg_iter
                .clone()
                .take(padding_bottom as usize * dimensions.width as usize),
        );

        // Sanity check
        if frame_pixels.next().is_some() {
            anyhow!("not enough pixels in frame");
        }

        display.update_pixels(pixels)?;
        Ok(())
    }
}
