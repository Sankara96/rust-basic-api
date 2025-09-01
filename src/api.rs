use std::{net::SocketAddr, sync::Arc, time::Duration};
use axum::{Router, routing::{get, post}, extract::State, Json, http::StatusCode};
use tower::{ServiceBuilder};
use tower_http::{trace::TraceLayer, compression::CompressionLayer, request_id::{MakeRequestUuid, SetRequestIdLayer}};
use crate::{model::{PlaceOrderRequest, PlaceOrderResponse, Side}, db::DbPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

pub fn build_app(state: AppState) -> Router {
    let shared = Arc::new(state);

    Router::new()
        .route("/healthz", get(health))
        .route("/orders", post(place_order))
        .route("/upload/accounts", post(super::upload::upload_accounts))
        .route("/openapi.json", get(serve_openapi))
        .with_state(shared)
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http())
                .timeout(Duration::from_secs(120))
        )
}

async fn health() -> &'static str { "Nuage Robo API is ready" }

async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(crate::openapi::ApiDoc::openapi())
}

#[axum::debug_handler]
async fn place_order(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PlaceOrderRequest>,
) -> Result<(StatusCode, Json<PlaceOrderResponse>), (StatusCode, String)> {
    if req.orders.is_empty() { return Err((StatusCode::BAD_REQUEST, "orders cannot be empty".into())); }
    for o in &req.orders {
        if !(o.side == Side::Buy || o.side == Side::Sell) || o.quantity <= 0.0 || o.account_id.is_empty() || o.security_id.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "invalid order line".into()));
        }
    }
    let request_id = req.request_id.unwrap_or_else(Uuid::new_v4);

    // TODO: call upstream exchange API (reused HTTP/2 client) here.

    Ok((StatusCode::ACCEPTED, Json(PlaceOrderResponse{
        request_id,
        status: "accepted".into(),
        accepted_count: req.orders.len(),
    })))
}
