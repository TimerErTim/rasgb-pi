use crate::display::Pixel;
use crate::frame::Frame;
use crate::web::api::error::{ResponseErrorExt, ResponseResult};
use crate::web::api::frame::data::FrameSubmitData;
use crate::web::state::WebServerContext;
use crate::web::FrameReceivedEvent;
use anyhow::anyhow;
use axum::http::StatusCode;
use base64::engine::{DecodePaddingMode, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use std::ops::Deref;
use std::sync::Arc;

pub async fn enqueue_frame(
    context: Arc<WebServerContext>,
    channel: Option<i8>,
    unix_micros: u128,
    data: FrameSubmitData,
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

    let event = FrameReceivedEvent {
        channel,
        unix_micros,
        frame,
    };
    context.control.on_frame_received.deref()(event)
        .map_err(|err| anyhow!(err).with_code(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::ACCEPTED)
}
