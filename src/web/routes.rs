use crate::web::api::{frames_router, meta_router};
use crate::web::state::WebServerContext;
use axum::Router;
use std::sync::Arc;
use tower_http::decompression::RequestDecompressionLayer;

pub fn build_routes(context: Arc<WebServerContext>) -> Router {
    Router::new()
        .merge(frames_router(&context))
        .layer(RequestDecompressionLayer::new())
        .with_state(context)
}
