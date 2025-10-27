// src/routes/product_routes.rs

use axum::{Router, routing::{get, post, put, delete}};
use crate::handlers::product_handlers::{
    get_all_products, create_product, find_product_by_id, update_product, delete_product
};
use crate::AppState;
use std::sync::Arc;

pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Rute untuk GET All dan POST Create
        .route("/", get(get_all_products).post(create_product))
        
    // Rute untuk GET Detail, PUT Edit, dan DELETE berdasarkan ID
    .route("/{id}", get(find_product_by_id)
               .put(update_product)
               .delete(delete_product))
}