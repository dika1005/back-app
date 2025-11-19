use axum::{Router, routing::{get, post, put, delete}, middleware::from_fn};
use crate::handlers::product::{
    create::create_product,
    get_all::get_all_products,
    get_by_id::find_product_by_id,
    update::update_product,
    delete::delete_product,
};
use crate::AppState;
use crate::middleware::auth::admin_auth_middleware;
use std::sync::Arc;

// Fungsi utama yang mengembalikan Router untuk rute Product
pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            get(get_all_products)
        )
        .route(
            "/create",
            post(create_product).route_layer(from_fn(admin_auth_middleware))
        )
        .route(
            "/{id}",
            get(find_product_by_id)
        )
        .route(
            "/{id}/update",
            put(update_product).route_layer(from_fn(admin_auth_middleware))
        )
        .route(
            "/{id}/delete",
            delete(delete_product).route_layer(from_fn(admin_auth_middleware))
        )
}