use serde::Deserialize;

#[derive(Deserialize)]
pub struct FrameSubmitData {
    pub unix_micros: u128,
    pub frame: FrameData,
}

#[derive(Deserialize)]
pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub pixels_b64: String,
}
