use crate::web::api::{frames_router, meta_router};
use crate::web::state::WebServerContext;
use axum::Router;
use std::sync::Arc;
use tower_http::decompression::RequestDecompressionLayer;

pub fn build_routes(context: Arc<WebServerContext>) -> Router {
    Router::new()
        .nest("/frame", frames_router(&context))
        .nest("/meta", meta_router(&context))
        .layer(RequestDecompressionLayer::new())
        .with_state(context)
}
