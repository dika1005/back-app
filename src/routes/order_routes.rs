use axum::{ routing::{ post, put, get }, Router, middleware::from_fn }; // <<< Tambahkan 'get' di sini
use std::sync::Arc;
use crate::AppState;
use crate::handlers::order_handlers;
use crate::middleware::auth::{ auth_user_middleware, admin_auth_middleware };

// FUNGSI INI HANYA UNTUK RUTE YANG HARUS DIAWALI DENGAN "/orders"
pub fn order_routes() -> Router<Arc<AppState>> {
    Router::new()
        // --- 1️⃣ POST /orders/checkout (User Auth) ---
        .route(
            "/checkout",
            post(order_handlers::checkout).route_layer(from_fn(auth_user_middleware)) // 🔒 hanya user login
        )
        // --- 2️⃣ PUT /orders/{id}/payment (Admin Only) ---
        .route(
            "/{id}/payment",
            put(order_handlers::process_payment).route_layer(from_fn(admin_auth_middleware)) // 🔒 hanya admin
        )
        // --- 3️⃣ GET /orders/{id}/status (DB Check - User Auth) ---
        .route(
            "/{id}/status",
            get(order_handlers::get_order_status_db).route_layer(from_fn(auth_user_middleware)) // 🔒 Cek status lokal
        )
        // --- 4️⃣ GET /orders/:id/midtrans-status (Midtrans API Check - User Auth) ---
        .route(
            "/{id}/midtrans-status",
            get(order_handlers::query_midtrans_status).route_layer(from_fn(auth_user_middleware)) // 🔒 Cek status Midtrans
        )
    // Rute /webhook/payment TELAH DIHAPUS DARI SINI (dan dipindahkan ke main.rs)
}
