use axum::{ routing::{ post, put }, Router, middleware::from_fn };
use std::sync::Arc;
use crate::AppState;
use crate::handlers::order_handlers;
use crate::middleware::auth::{ auth_user_middleware, admin_auth_middleware }; // pastikan ini sesuai nama middleware kamu

pub fn order_routes() -> Router<Arc<AppState>> {
    Router::new()
        // --- 1️⃣ Checkout (User Auth) ---
        .route(
            "/checkout",
            post(order_handlers::checkout).route_layer(from_fn(auth_user_middleware)) // 🔒 hanya user login
        )
        // --- 2️⃣ Proses Pembayaran (Admin Only) ---
        .route(
            "/orders/{id}/payment",
            put(order_handlers::process_payment).route_layer(from_fn(admin_auth_middleware)) // 🔒 hanya admin
        )

        // --- 3️⃣ Webhook dari Payment Gateway ---
        .route(
            "/webhook/payment",
            post(order_handlers::webhook_payment) // 🌐 tanpa middleware (dibuka untuk sistem gateway)
        )
}
