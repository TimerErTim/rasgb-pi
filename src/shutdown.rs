use crate::context::RasGBContext;

pub async fn shutdown(context: RasGBContext) {
    eprintln!("gracefully quitting...");
    context.shutdown_token.cancel();
}
