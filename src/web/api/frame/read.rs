use crate::web::api::error::ResponseResult;
use crate::web::state::WebServerContext;
use crate::web::FrameSupersededCheckEvent;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use std::ops::Deref;
use std::sync::Arc;

pub async fn check_superseded_frame(
    context: Arc<WebServerContext>,
    channel: Option<i8>,
    unix_micros: u128,
) -> ResponseResult<Response> {
    let is_obsolete =
        context.control.on_frame_superseded_check.deref()(FrameSupersededCheckEvent {
            channel,
            unix_micros,
        });

    let mut response = Response::builder()
        .header("Display-Width", context.control.display_width.to_string())
        .header("Display-Height", context.control.display_height.to_string())
        .header("Display-FPS", context.control.display_fps.to_string())
        .body(Body::empty())?;
    *response.status_mut() = if is_obsolete {
        StatusCode::NOT_MODIFIED
    } else {
        StatusCode::ACCEPTED
    };

    Ok(response)
}
