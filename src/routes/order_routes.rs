use crate::AppState;
use axum::{
    Router,
    middleware::from_fn,
    routing::{get, post, put},
};
use std::sync::Arc;

use crate::handlers::order::{
    checkout::checkout, process_payment::process_payment, query_midtrans::query_midtrans_status,
    status_db::get_order_status_db,
};

use crate::middleware::auth::{admin_auth_middleware, auth_user_middleware};

pub fn order_routes() -> axum::Router<Arc<AppState>> {
    Router::new()
        .route(
            "/checkout",
            post(checkout).route_layer(from_fn(auth_user_middleware)),
        )
        .route(
            "/{id}/payment",
            put(process_payment).route_layer(from_fn(admin_auth_middleware)),
        )
        .route(
            "/{id}/status",
            get(get_order_status_db).route_layer(from_fn(auth_user_middleware)),
        )
        .route(
            "/{id}/midtrans-status",
            get(query_midtrans_status).route_layer(from_fn(auth_user_middleware)),
        )
}
