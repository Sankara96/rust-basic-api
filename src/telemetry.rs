use tracing_subscriber::{EnvFilter, fmt};

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "place_order=info,axum=info,tower_http=info".into());
    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}
