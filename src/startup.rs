use crate::context::RasGBContext;

pub async fn startup() -> RasGBContext {
    RasGBContext::testing()
}
