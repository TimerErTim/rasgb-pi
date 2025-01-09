use crate::web::state::WebServerContext;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

mod error;
mod frame;
mod meta;

pub fn frames_router(context: &WebServerContext) -> Router<Arc<WebServerContext>> {
    let pixel_count = context.control.display_width * context.control.display_height;
    let max_frame_size = (pixel_count * 5) as usize; // Account for base64 encoding

    Router::new()
        .route("/", post(frame::post_frame))
        .layer(DefaultBodyLimit::max(max_frame_size + 1024))
}

pub fn meta_router(_context: &WebServerContext) -> Router<Arc<WebServerContext>> {
    Router::new().route("/", get(meta::get_meta))
}
