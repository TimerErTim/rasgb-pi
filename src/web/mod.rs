use crate::frame::Frame;
use crate::web::routes::build_routes;
use crate::web::state::WebServerContext;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

mod api;
pub mod routes;
pub mod state;

pub struct WebServerConfig {
    pub socket: SocketAddr,
    pub shutdown_signal: Option<Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>>,
}

pub struct WebServerControl {
    pub display_width: u32,
    pub display_height: u32,
    pub display_fps: f64,
    pub on_frame_received: Box<dyn Fn(FrameReceivedEvent) -> Result<(), String> + Send + Sync>,
}

pub struct FrameReceivedEvent {
    pub channel: Option<u8>,
    pub unix_micros: u128,
    pub frame: Frame,
}

pub async fn run_server(mut config: WebServerConfig, control: WebServerControl) {
    let shutdown_signal = config.shutdown_signal.take();
    let listener = tokio::net::TcpListener::bind(config.socket).await.unwrap();

    let context = Arc::new(WebServerContext { config, control });

    let server_future = axum::serve(listener, build_routes(Arc::clone(&context)));
    eprintln!("listening on {}", context.config.socket);
    let server_result = if let Some(shutdown_signal) = shutdown_signal {
        server_future.with_graceful_shutdown(shutdown_signal).await
    } else {
        server_future.await
    };
    match server_result {
        Ok(()) => eprintln!("server shutdown {}", context.config.socket),
        Err(e) => eprintln!("server error: {}", e),
    };
}
