// src/routes/kategori_routes.rs

use axum::{ Router, routing::{ get, post, put, delete }, middleware::from_fn };
use std::sync::Arc;
use crate::AppState;

use crate::handlers::category::{
    get_all::get_all_categories,
    create::create_category,
    get_by_id::get_category_by_id,
    update::update_category,
    delete::delete_category,
};

use crate::middleware::auth::{ admin_auth_middleware };

pub fn category_routes() -> axum::Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            get(get_all_categories)
        )
        .route(
            "/create",
            post(create_category).route_layer(from_fn(admin_auth_middleware))
        )
        .route(
            "/{id}",
            get(get_category_by_id)
        )
        .route(
            "/{id}/update",
            put(update_category).route_layer(from_fn(admin_auth_middleware))
        )
        .route(
            "/{id}/delete",
            delete(delete_category).route_layer(from_fn(admin_auth_middleware))
        )
}
