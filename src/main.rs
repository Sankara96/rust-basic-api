// Keep your other modules in the repo, but don't reference them now.
mod model;
// mod api;
// mod db;
// mod upload;
// mod openapi;
// mod telemetry;

use std::net::SocketAddr;
use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // telemetry::init_tracing();
    // let db = db::DbPool::from_env()?;
    // let state = api::AppState { db };
    // let app: Router = api::build_app(state);

    // Minimal app with only /healthz
    let app = Router::new()
        .route("/healthz", get(|| async { "Nuage Robo API is Ready" }));

    // HTTP inside cluster; TLS is terminated at Ingress
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    println!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
