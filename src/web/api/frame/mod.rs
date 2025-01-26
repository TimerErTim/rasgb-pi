mod data;

use crate::display::Pixel;
use crate::frame::Frame;
use crate::web::api::error::{ResponseErrorExt, ResponseResult};
use crate::web::state::WebServerContext;
use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use base64::engine::{DecodePaddingMode, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use std::ops::Deref;
use std::sync::Arc;

pub async fn post_frame(
    State(context): State<Arc<WebServerContext>>,
    Json(data): Json<data::FrameSubmitData>,
) -> ResponseResult<StatusCode> {
    let mut pixel_bytes = Vec::with_capacity((data.frame.width * data.frame.height * 3) as usize);
    let base64_engine = base64::engine::general_purpose::GeneralPurpose::new(
        &alphabet::STANDARD,
        GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
    );
    base64_engine
        .decode_vec(data.frame.pixels_b64, &mut pixel_bytes)
        .map_err(|err| err.with_code(StatusCode::UNPROCESSABLE_ENTITY))?;

    let frame = Frame::new(
        data.frame.width,
        data.frame.height,
        pixel_bytes
            .as_slice()
            .chunks_exact(3)
            .map(|chunk| Pixel {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
            })
            .collect(),
    )
    .map_err(|err| err.with_code(StatusCode::NOT_ACCEPTABLE))?;

    context.control.on_frame_received.deref()(data.unix_micros, frame)
        .map_err(|err| anyhow!(err).with_code(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::ACCEPTED)
}
