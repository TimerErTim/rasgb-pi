use serde::Serialize;

#[derive(Serialize)]
pub struct MetaData {
    pub display: DisplayData,
}

#[derive(Serialize)]
pub struct DisplayData {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}
