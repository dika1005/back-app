use axum::{ routing::{ post, put, get }, Router, middleware::from_fn };
use std::sync::Arc;
use crate::AppState;

use crate::handlers::order::{
    checkout::checkout,
    process_payment::process_payment,
    status_db::get_order_status_db,
    query_midtrans::query_midtrans_status,
};

use crate::middleware::auth::{ auth_user_middleware, admin_auth_middleware };

pub fn order_routes() -> axum::Router<Arc<AppState>> {
    Router::new()
        .route(
            "/checkout",
            post(checkout).route_layer(from_fn(auth_user_middleware))
        )
        .route(
            "/{id}/payment",
            put(process_payment).route_layer(from_fn(admin_auth_middleware))
        )
        .route(
            "/{id}/status",
            get(get_order_status_db).route_layer(from_fn(auth_user_middleware))
        )
        .route(
            "/{id}/midtrans-status",
            get(query_midtrans_status).route_layer(from_fn(auth_user_middleware))
        )
}
