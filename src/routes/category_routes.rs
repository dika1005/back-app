// src/routes/kategori_routes.rs

use crate::AppState;
use axum::{
    Router,
    middleware::from_fn,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::handlers::category::{
    create::create_category, delete::delete_category, get_all::get_all_categories,
    get_by_id::get_category_by_id, update::update_category,
};

use crate::middleware::auth::admin_auth_middleware;

pub fn category_routes() -> axum::Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all_categories))
        .route(
            "/create",
            post(create_category).route_layer(from_fn(admin_auth_middleware)),
        )
        .route("/{id}", get(get_category_by_id))
        .route(
            "/{id}/update",
            put(update_category).route_layer(from_fn(admin_auth_middleware)),
        )
        .route(
            "/{id}/delete",
            delete(delete_category).route_layer(from_fn(admin_auth_middleware)),
        )
}
