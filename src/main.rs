use axum::{Router, routing::get, serve};
use dotenvy::dotenv;
use sqlx::{MySql, Pool};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, AllowOrigin, AllowMethods, AllowHeaders};

mod db;
mod models;
mod dtos;
mod routes;
mod handlers;
mod utils;
mod middleware;

use routes::{auth_routes::auth_routes, user_routes::user_routes, category_routes::category_routes, product_routes::product_routes};

// ========================
// Struct Global AppState
// ========================
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
}

// ========================
// Main Function
// ========================
#[tokio::main]
async fn main() {
    dotenv().ok();

    // --- 1. Koneksi Database ---
    let database_url = env::var("DATABASE_URL").expect("❌ DATABASE_URL harus diatur di .env");
    let db_pool = Pool::<MySql>::connect(&database_url)
        .await
        .expect("❌ Gagal konek ke MySQL");

    println!("✅ Connected to MySQL successfully!");

    // --- 2. Shared State ---
    let shared_state = Arc::new(AppState { db: db_pool });

    // --- 3. Setup Router ---
    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/auth", auth_routes())
        .nest("/user", user_routes())
        .nest("/categories", category_routes())
        .nest("/products", product_routes())
        .with_state(shared_state)
        .layer(cors_layer()); // tambahkan CORS layer

    // --- 4. Jalankan Server ---
    let host = env::var("BIND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("BIND_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3001);

    let addr = SocketAddr::from((host.parse::<std::net::IpAddr>().unwrap(), port));
    println!("🚀 Server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app.into_make_service()).await.unwrap();
}

// ========================
// Handlers & Utils Lokal
// ========================

async fn root_handler() -> &'static str {
    "Server connected to MySQL successfully!"
}

// CORS Setup biar modular & bersih
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
}
