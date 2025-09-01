use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::place_order,
        crate::upload::upload_accounts,
        crate::api::serve_openapi,
        crate::api::health
    ),
    components(
        schemas(
            crate::model::PlaceOrderRequest,
            crate::model::PlaceOrderResponse,
            crate::model::OrderLine,
            crate::model::Side
        )
    ),
    tags(
        (name="orders", description="Order placement"),
        (name="upload", description="CSV upload")
    )
)]
pub struct ApiDoc;
