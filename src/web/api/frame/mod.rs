mod data;
mod read;
mod write;

use crate::display::Pixel;
use crate::frame::Frame;
use crate::web::api::error::{ResponseErrorExt, ResponseResult};
use crate::web::api::frame::read::check_superseded_frame;
use crate::web::api::frame::write::enqueue_frame;
use crate::web::state::WebServerContext;
use crate::web::FrameReceivedEvent;
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use base64::{alphabet, Engine};
use std::ops::Deref;
use std::sync::Arc;

pub async fn post_frame_with_channel(
    State(context): State<Arc<WebServerContext>>,
    Path((unix_micros, channel)): Path<(u128, i8)>,
    Json(data): Json<data::FrameSubmitData>,
) -> ResponseResult<StatusCode> {
    enqueue_frame(context, Some(channel), unix_micros, data).await
}

pub async fn post_frame(
    State(context): State<Arc<WebServerContext>>,
    Path(unix_micros): Path<u128>,
    Json(data): Json<data::FrameSubmitData>,
) -> ResponseResult<StatusCode> {
    enqueue_frame(context, None, unix_micros, data).await
}

pub async fn head_frame_with_channel(
    State(context): State<Arc<WebServerContext>>,
    Path((unix_micros, channel)): Path<(u128, i8)>,
) -> ResponseResult<Response> {
    check_superseded_frame(context, Some(channel), unix_micros).await
}

pub async fn head_frame(
    State(context): State<Arc<WebServerContext>>,
    Path(unix_micros): Path<u128>,
) -> ResponseResult<Response> {
    check_superseded_frame(context, None, unix_micros).await
}
