use axum::{Router, routing::{get, post, put}, serve};
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

// IMPORT SEMUA RUTE DAN HANDLER WEBHOOK
use routes::{auth_routes::auth_routes, user_routes::user_routes, category_routes::category_routes, product_routes::product_routes, order_routes::order_routes};
use handlers::order_handlers::webhook_payment; // <<< IMPORT HANDLER WEBHOOK DARI SINI >>>

// ========================
// Struct Global AppState
// ========================
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub midtrans_server_key: String,
    pub midtrans_client_key: String,
    pub midtrans_base_url: String,
}

// ========================
// Main Function
// ========================
#[tokio::main]
async fn main() {
    dotenv().ok();
    
    // --- 1. Koneksi Database ---
    let db_pool = db::init_db().await;
    
    // 🧩 2. Ambil variabel Midtrans dari .env
    let midtrans_server_key = env::var("MIDTRANS_SERVER_KEY").expect("MIDTRANS_SERVER_KEY belum diset di .env");
    let midtrans_client_key = env::var("MIDTRANS_CLIENT_KEY").expect("MIDTRANS_CLIENT_KEY belum diset di .env");
    let midtrans_base_url = env::var("MIDTRANS_BASE_URL").unwrap_or_else(|_| "https://api.sandbox.midtrans.com".to_string());
    
    let shared_state = Arc::new(AppState {
        db: db_pool,
        midtrans_server_key,
        midtrans_client_key,
        midtrans_base_url,
    });

    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/auth", auth_routes())
        .nest("/user", user_routes())
        .nest("/categories", category_routes())
        .nest("/products", product_routes())
        
        // --- DAFTAR RUTE TRANSAKSI (Membutuhkan Awalan /orders) ---
        .nest("/orders", order_routes()) 
        
        // <<< SOLUSI UNTUK 404 WEBHOOK >>>
        // Rute ini HARUS berada di root ("/") agar alamatnya menjadi POST /webhook/payment
        .route("/webhook/payment", post(webhook_payment))
        
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
    "Server connected to MySQL successfully! Available endpoints: /auth, /user, /categories, /products, /orders."
}

// CORS Setup biar modular & bersih
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
}
