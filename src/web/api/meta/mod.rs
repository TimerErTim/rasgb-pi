mod data;

use crate::web::api::error::ResponseResult;
use crate::web::api::meta::data::{DisplayData, MetaData};
use crate::web::state::WebServerContext;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

pub async fn get_meta(
    State(context): State<Arc<WebServerContext>>,
) -> ResponseResult<Json<MetaData>> {
    let meta_data = MetaData {
        display: DisplayData {
            width: context.control.display_width,
            height: context.control.display_height,
            fps: context.control.display_fps,
        },
    };
    Ok(Json(meta_data))
}
