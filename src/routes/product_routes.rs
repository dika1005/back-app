use axum::{Router, routing::{get, post, put}};
use crate::handlers::product_handlers::{
    create_product,
    get_all_products,
    find_product_by_id,
    update_product,
    delete_product,
};
use crate::AppState;
use std::sync::Arc;

// Fungsi utama yang mengembalikan Router untuk rute Product
pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        // GET all products and POST create (admin)
        .route("/", get(get_all_products).post(create_product))
        // Detail, update, delete
    .route("/{id}", get(find_product_by_id).put(update_product).delete(delete_product))
}