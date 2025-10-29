use axum::{ 
    extract::{ Path, State, Json }, 
    response::{ IntoResponse, Response },
    http::StatusCode,
};
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
// Import DTO dan Model Order
use crate::dtos::order::{NewOrderDto, OrderDetail, Order}; 
use crate::models::user::User;
// Import Guard/Middleware
use crate::middleware::auth::{AuthUser, AdminAuth}; 

// Type alias untuk Result yang konsisten
type HandlerResult<T> = Result<T, (StatusCode, String)>;

// Fungsi pembantu untuk membuat error response 500
fn internal_server_error(e: sqlx::Error) -> (StatusCode, String) {
    eprintln!("Database Error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan internal pada server.".to_string())
}

// --- 1. POST /checkout (CREATE ORDER) - REQUIRES AUTHUSER ---
pub async fn checkout(
    State(state): State<Arc<AppState>>,
    // AuthUser Guard: Mengambil email dari token yang terotentikasi
    auth_user: AuthUser, 
    Json(new_order_dto): Json<NewOrderDto>
) -> HandlerResult<impl IntoResponse> {
    // Cari user di DB berdasarkan email yang ada di token
    let user_record = match User::find_by_email(&state.db, &auth_user.email).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err((StatusCode::UNAUTHORIZED, "User tidak ditemukan".to_string())),
        Err(e) => return Err(internal_server_error(e)),
    };

    // Panggil logika transaksi create_order dari model (pakai user_id)
    match Order::create_order(&state.db, user_record.id, new_order_dto).await {
        Ok(order_id) => {
            // Berhasil membuat Order
            Ok((
                StatusCode::CREATED,
                Json(
                    ApiResponse::success_data_with_message(
                        format!("Checkout berhasil. Order ID: {}", order_id),
                        order_id
                    )
                ),
            ))
        }
        Err(sqlx::Error::RowNotFound) => {
             // Produk tidak ditemukan saat validasi di dalam transaksi
            Err((
                StatusCode::BAD_REQUEST,
                "Gagal checkout. Salah satu produk dalam pesanan tidak valid atau tidak ditemukan.".to_string(),
            ))
        }
        Err(e) => {
            // Kesalahan database umum atau transaksi gagal
            Err(internal_server_error(e))
        }
    }
}

// --- 2. PUT /orders/:id/payment (PROCESS PAYMENT) - REQUIRES ADMINAUTH (Simulasi Webhook/Admin) ---
pub async fn process_payment(
    State(state): State<Arc<AppState>>,
    // AdminAuth Guard: Hanya admin atau sistem webhook yang boleh mengubah status
    AdminAuth(_): AdminAuth, 
    Path(order_id): Path<i64>,
    // Payload sederhana untuk menentukan status pembayaran
    Json(payload): Json<serde_json::Value> 
) -> HandlerResult<impl IntoResponse> {
    
    // --- Logika Simulasi Pembayaran ---
    // Cek apakah payload menyatakan "success"
    let is_success = payload["status"]
        .as_str()
        .map(|s| s.to_lowercase() == "success")
        .unwrap_or(false);

    // Panggil logika update status dari model
    match Order::process_payment(&state.db, order_id, is_success).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                let status_msg = if is_success { "PAID" } else { "FAILED" };
                Ok((
                    StatusCode::OK, 
                    Json(ApiResponse::<()>::success(&format!("Status Order {} diperbarui menjadi {}", order_id, status_msg)))
                ))
            } else {
                // Order tidak ditemukan atau statusnya sudah bukan 'PENDING'
                Err((StatusCode::NOT_FOUND, "Order tidak ditemukan atau status sudah diproses.".to_string()))
            }
        }
        Err(e) => {
            // Kesalahan database
            Err(internal_server_error(e))
        }
    }
}

// Catatan: Anda juga bisa menambahkan handler GET /orders/:id untuk melihat detail order