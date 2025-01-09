use tokio::signal::unix::SignalKind;
use tokio::task::JoinSet;

pub async fn exit_signal() {
    let mut exit_signal_set = JoinSet::new();

    exit_signal_set.spawn(ctrl_c());
    exit_signal_set.spawn(sig_term());

    exit_signal_set.join_next().await;
}

async fn ctrl_c() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn sig_term() {
    if let Ok(mut signal) = tokio::signal::unix::signal(SignalKind::terminate()) {
        signal.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}
