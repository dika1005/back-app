use axum::{ Router, routing::{ get, post }, serve };
use dotenvy::dotenv;
use sqlx::{ MySql, Pool };
use std::{ env, net::SocketAddr, sync::Arc };
use tokio::net::TcpListener;
use tower_http::cors::{ AllowHeaders, AllowMethods, AllowOrigin, CorsLayer };
use utoipa::OpenApi;

mod db;
mod dtos;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;

// IMPORT SEMUA RUTE DAN HANDLER WEBHOOK
use crate::handlers::order::webhook::webhook_payment;
use routes::{
    auth_routes::auth_routes,
    category_routes::category_routes,
    order_routes::order_routes,
    product_routes::product_routes,
    user_routes::user_routes,
    chatbot_routes::chatbot_routes,
}; // <<< IMPORT HANDLER WEBHOOK DARI SINI >>>

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
// OpenAPI Documentation
// ========================
#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth endpoints
        handlers::auth::register::register_handler,
        handlers::auth::login::login_handler,
        
        // Product endpoints
        handlers::product::get_all::get_all_products,
        handlers::product::get_by_id::find_product_by_id,
        handlers::product::create::create_product,
        
        // Category endpoints
        handlers::category::get_all::get_all_categories,
        handlers::category::create::create_category,
        
        // Order endpoints
        handlers::order::checkout::checkout,
        
        // Chatbot
        handlers::chatbot::recommend::chatbot_recommend,
    ),
    components(
        schemas(
            // Auth DTOs
            dtos::auth::register::RegisterRequest,
            dtos::auth::register::RegisterResponse,
            dtos::auth::register::UserData,
            dtos::auth::login::LoginRequest,
            dtos::auth::login::LoginResponse,
            dtos::auth::login::UserLoginData,
            dtos::auth::update_role::UpdateRoleRequest,
            
            // Product DTOs
            dtos::product::NewRodProductDto,
            dtos::product::RodProductDetail,
            dtos::product::RodProduct,
            
            // Category DTOs
            dtos::category::KategoriDto,
            dtos::category::NewKategoriDto,
            
            // Order DTOs
            dtos::order::OrderItem,
            dtos::order::NewOrderDto,
            
            // User DTOs
            dtos::user::UpdateProfile,
            
            // Chatbot DTOs
            dtos::chatbot::ChatRequest,
            dtos::chatbot::ChatResponse,
            
            // Pagination
            dtos::pagination::PaginationParams,
            dtos::pagination::PaginationMeta,
            
            // Generic Response
            utils::api_response::ApiResponse<String>,
        )
    ),
    tags(
        (name = "auth", description = "Authentication and authorization endpoints"),
        (name = "products", description = "Fishing rod product management"),
        (name = "categories", description = "Product category management"),
        (name = "orders", description = "Order and payment management"),
        (name = "chatbot", description = "AI-powered product recommendations"),
        (name = "user", description = "User profile management")
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Fishing Rod E-commerce API",
        version = "1.0.0",
        description = "REST API untuk toko joran pancing dengan fitur:\n\
        - Autentikasi JWT dan Google OAuth\n\
        - Manajemen produk dan kategori\n\
        - Sistem order dengan Midtrans payment gateway\n\
        - AI chatbot untuk rekomendasi produk (Groq LLM)\n\
        - Role-based access control (admin/user)",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    servers(
        (url = "http://127.0.0.1:3001", description = "Development server")
    )
)]
struct ApiDoc;

// Security scheme untuk JWT authentication
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("Enter JWT token from login response"))
                        .build(),
                ),
            );
        }
    }
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
    let midtrans_server_key = env
        ::var("MIDTRANS_SERVER_KEY")
        .expect("MIDTRANS_SERVER_KEY belum diset di .env");
    let midtrans_client_key = env
        ::var("MIDTRANS_CLIENT_KEY")
        .expect("MIDTRANS_CLIENT_KEY belum diset di .env");
    let midtrans_base_url = env
        ::var("MIDTRANS_BASE_URL")
        .unwrap_or_else(|_| "https://api.sandbox.midtrans.com".to_string());

    let shared_state = Arc::new(AppState {
        db: db_pool,
        midtrans_server_key,
        midtrans_client_key,
        midtrans_base_url,
    });

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api-docs/openapi.json", get(openapi_json))
        .nest("/auth", auth_routes())
        .nest("/user", user_routes())
        .nest("/categories", category_routes())
        .nest("/products", product_routes())
        .nest("/orders", order_routes())
        .route("/webhook/payment", post(webhook_payment))
        .nest("/chatbot", chatbot_routes())
        .layer(cors_layer()) // tambahkan CORS layer
        .with_state(shared_state);
    
    // Note: OpenAPI JSON available at /api-docs/openapi.json
    // Use Swagger Editor (https://editor.swagger.io) or Postman to view the documentation

    // --- 4. Jalankan Server ---
    let host = env::var("BIND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env
        ::var("BIND_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3001);

    let addr = SocketAddr::from((host.parse::<std::net::IpAddr>().unwrap(), port));
    println!("🚀 Server running at http://{}", addr);
    println!("📚 OpenAPI JSON: http://{}/api-docs/openapi.json", addr);
    println!("💡 View docs at: https://editor.swagger.io (paste the JSON URL)");

    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app.into_make_service()).await.unwrap();
}

// ========================
// Handlers & Utils Lokal
// ========================

async fn root_handler() -> &'static str {
    "Server connected to MySQL successfully! Available endpoints: /auth, /user, /categories, /products, /orders. API Docs: /api-docs/openapi.json"
}

async fn openapi_json() -> axum::Json<utoipa::openapi::OpenApi> {
    axum::Json(ApiDoc::openapi())
}

// CORS Setup biar modular & bersih
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
}
