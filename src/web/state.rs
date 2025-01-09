use crate::frame::gen::web::WebQueriedFrameGeneratorConfig;
use crate::frame::Frame;
use crate::web::{WebServerConfig, WebServerControl};
use std::future::Future;
use std::net::SocketAddr;

pub struct WebServerContext {
    pub config: WebServerConfig,
    pub control: WebServerControl,
}
