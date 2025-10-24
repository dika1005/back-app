use axum::{Router, routing::get, serve};
use dotenvy::dotenv;
use sqlx::{MySql, Pool};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, AllowOrigin, AllowMethods, AllowHeaders};

mod db;
mod routes;
mod handlers;
mod utils;
mod middleware;


use routes::{
    auth_routes::auth_routes,
    user_routes::user_routes,
};


// --- AppState (Global State) ---
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // --- Koneksi ke Database ---
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL harus diatur di .env");
    let pool = Pool::<MySql>::connect(&database_url)
        .await
        .expect("Gagal konek ke MySQL");

    println!("✅ Connected to MySQL successfully!");

    // --- Bungkus ke Arc<AppState> biar thread-safe dan bisa dishare ---
    let shared_state = Arc::new(AppState { db: pool });

    // --- Buat Router dan gabungkan semua routes ---
    let app = Router::new()
        .route("/", get(|| async { "Server connected to MySQL successfully!" }))
        .nest("/auth", auth_routes()) // <--- gabungin route auth di sini
        .nest("/user", user_routes()) // <--- gabungin route user di sini
        .with_state(shared_state.clone()); // kasih state ke semua route

    // CORS disabled temporarily to avoid layering type mismatch.
    // If needed, re-enable after dependency version alignment.

    // --- Jalankan Server ---
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
