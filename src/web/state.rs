use crate::web::{WebServerConfig, WebServerControl};

pub struct WebServerContext {
    pub config: WebServerConfig,
    pub control: WebServerControl,
}
