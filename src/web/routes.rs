use crate::web::api::{frames_router, meta_router};
use crate::web::state::WebServerContext;
use axum::Router;
use std::sync::Arc;

pub fn build_routes(context: Arc<WebServerContext>) -> Router {
    Router::new()
        .nest("/frame", frames_router(&context))
        .nest("/meta", meta_router(&context))
        .with_state(context)
}
