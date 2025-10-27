// src/routes/kategori_routes.rs

use axum::{Router, routing::{get, post, put, delete}};
use crate::handlers::category_handlers::{
    get_all_categories, create_category, get_category_by_id, update_category, delete_category
};
use crate::AppState;
use std::sync::Arc;

pub fn category_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Rute untuk GET All dan POST Create
        .route("/", get(get_all_categories).post(create_category))
        
    // Rute untuk GET Detail, PUT Edit, dan DELETE berdasarkan ID
    .route("/{id}", get(get_category_by_id)
               .put(update_category)
               .delete(delete_category))
}