use std::fmt::{Debug, Display, Formatter};

use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::runtime::Handle;

pub type ResponseResult<T> = Result<T, ResponseError>;

pub struct ResponseError(StatusCode, anyhow::Error);

impl ResponseError {
    pub fn from_response(response: impl IntoResponse) -> Self {
        let response = response.into_response();
        let status = response.status();
        let content = match Handle::current()
            .block_on(axum::body::to_bytes(response.into_body(), usize::MAX))
        {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => "".to_string(),
        };
        ResponseError(status, anyhow!(content))
    }

    pub fn status(&self) -> StatusCode {
        self.0.clone()
    }
}

pub(crate) trait ResponseErrorExt {
    fn with_code(self, code: impl Into<StatusCode>) -> ResponseError;
}

impl Debug for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.1, f)
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.1, f)
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        (self.0, format!("{:?}", self.1)).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for ResponseError {
    fn from(value: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, value.into())
    }
}

impl<E: Into<ResponseError>> ResponseErrorExt for E {
    fn with_code(self, code: impl Into<StatusCode>) -> ResponseError {
        let mut error = self.into();
        error.0 = code.into();
        error
    }
}
