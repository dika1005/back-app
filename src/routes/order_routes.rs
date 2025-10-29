use axum::{
    routing::{post, put},
    Router,
    middleware::from_fn,
};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::order_handlers; // Import handler yang sudah dibuat
use crate::middleware::auth::{AuthUser, AdminAuth}; // Import middleware

// Fungsi utama yang mengembalikan Router untuk rute Order
pub fn order_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Rute untuk melakukan checkout (membuat order baru)
        // POST /checkout
        .route("/checkout", post(order_handlers::checkout))
        
        // Rute untuk memproses pembayaran (mengubah status order)
        // PUT /orders/:id/payment
        // Rute ini memerlukan hak akses Admin/Sistem (AdminAuth)
    .route("/{id}/payment", put(order_handlers::process_payment))
}