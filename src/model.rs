use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Side { Buy, Sell }

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct OrderLine {
    pub account_id: String,
    pub security_id: String,
    pub side: Side,
    pub quantity: f64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct PlaceOrderRequest {
    /// Client idempotency key (server will mint if absent)
    pub request_id: Option<Uuid>,
    pub asof: Option<String>,
    pub orders: Vec<OrderLine>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct PlaceOrderResponse {
    pub request_id: Uuid,
    pub status: String,
    pub accepted_count: usize,
}
