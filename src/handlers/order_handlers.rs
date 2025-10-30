use axum::{
    extract::{Path, State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use crate::{
    AppState,
    utils::ApiResponse,
    dtos::order::{NewOrderDto, Order},
    models::user::User,
    middleware::auth::{AuthUser, AdminAuth},
};
use reqwest::Client;
use serde_json::json;
// PERBAIKAN: Import Engine untuk mengganti base64::encode() yang deprecated
use base64::{Engine as _, engine::general_purpose}; 

// Type alias untuk Result yang konsisten
type HandlerResult<T> = Result<T, (StatusCode, String)>;

// Fungsi pembantu untuk membuat error response 500
fn internal_server_error(e: sqlx::Error) -> (StatusCode, String) {
    eprintln!("Database Error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan internal pada server.".to_string())
}

// --- 1️⃣ POST /checkout ---
// Membuat order dan memanggil API Midtrans Snap
pub async fn checkout(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(new_order_dto): Json<NewOrderDto>,
) -> HandlerResult<impl IntoResponse> {
    // 🔍 Cari user di DB berdasarkan email token
    let user_record = match User::find_by_email(&state.db, &auth_user.email).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err((StatusCode::UNAUTHORIZED, "User tidak ditemukan".to_string())),
        Err(e) => return Err(internal_server_error(e)),
    };

    // 🧾 Buat order di database
    match Order::create_order(&state.db, user_record.id, new_order_dto).await {
        Ok(order_id) => {
            let client = Client::new();

            // Payload Midtrans Snap API
            let payload = json!({
                "transaction_details": {
                    "order_id": order_id.to_string(),
                    "gross_amount": 100000 // TODO: ganti dengan total harga sebenarnya
                },
                "customer_details": {
                    "first_name": user_record.name,
                    "email": user_record.email
                },
                "credit_card": { "secure": true }
            });

            // Gunakan base URL dari .env
            let url = format!("{}/snap/v1/transactions", state.midtrans_base_url);

            // Persiapan Otentikasi Midtrans (Basic Auth: ServerKey:)
            let server_key = &state.midtrans_server_key;
            let auth_string = format!("{}:", server_key); // Format Midtrans: ServerKey diikuti titik dua
            
            // PERBAIKAN: Menggunakan Engine yang tidak deprecated
            let encoded_auth = general_purpose::STANDARD.encode(auth_string);

            // Debug: tampilkan URL dan key (dengan masking)
            let masked_key = if server_key.len() > 8 {
                format!("{}*****", &server_key[..8])
            } else {
                "*****".to_string()
            };
            eprintln!("[midtrans] URL: {}", url);
            eprintln!("[midtrans] Server key (masked): {}", masked_key);
            eprintln!("[midtrans] Auth Header (Partial): Basic {}...", &encoded_auth[..15]);


            // 🔐 Request ke Midtrans Snap API dengan header Authorization manual
            let resp = client
                .post(&url)
                // Mengganti basic_auth() yang bermasalah dengan header Authorization eksplisit
                .header("Authorization", format!("Basic {}", encoded_auth)) 
                .json(&payload)
                .send()
                .await;

            match resp {
                Ok(r) => {
                    // PERBAIKAN E0382: Simpan status sebelum memanggil r.text()
                    let status = r.status(); 
                    
                    if status.is_success() {
                        let data: serde_json::Value = r.json().await.unwrap_or_default();
                        let redirect_url =
                            data["redirect_url"].as_str().unwrap_or("").to_string();

                        Ok((
                            StatusCode::CREATED,
                            Json(ApiResponse::success_data_with_message(
                                "Checkout berhasil. Silakan lanjutkan pembayaran melalui Midtrans."
                                    .to_string(),
                                json!({
                                    "order_id": order_id,
                                    "payment_url": redirect_url
                                }),
                            )),
                        ))
                    } else {
                        // r.text().await mengambil ownership 'r'
                        let error_body = r.text().await.unwrap_or_else(|_| "Failed to read response body".to_string());
                        
                        // Menggunakan variabel 'status' yang sudah disimpan
                        eprintln!("Midtrans Error Status: {}", status);
                        eprintln!("Midtrans Error Body: {}", error_body);
                        Err((
                            StatusCode::BAD_GATEWAY,
                            format!("Gagal membuat transaksi di Midtrans. Detail: {}", error_body),
                        ))
                    }
                }
                Err(e) => {
                    eprintln!("Midtrans Request Error: {}", e);
                    Err((
                        StatusCode::BAD_GATEWAY,
                        "Tidak dapat terhubung ke Midtrans.".to_string(),
                    ))
                }
            }
        }
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::BAD_REQUEST,
            "Gagal checkout. Salah satu produk tidak valid.".to_string(),
        )),
        Err(e) => Err(internal_server_error(e)),
    }
}

// --- 2️⃣ PUT /orders/:id/payment (Admin Update Status) ---
pub async fn process_payment(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Path(order_id): Path<i64>,
    Json(payload): Json<serde_json::Value>,
) -> HandlerResult<impl IntoResponse> {
    let is_success = payload["status"]
        .as_str()
        .map(|s| s.to_lowercase() == "success")
        .unwrap_or(false);

    match Order::process_payment(&state.db, order_id, is_success).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                let status_msg = if is_success { "PAID" } else { "FAILED" };
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success(&format!(
                        "Status Order {} diperbarui menjadi {}",
                        order_id, status_msg
                    ))),
                ))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    "Order tidak ditemukan atau sudah diproses.".to_string(),
                ))
            }
        }
        Err(e) => Err(internal_server_error(e)),
    }
}

// --- 3️⃣ POST /webhook/payment (Dari Midtrans) ---
pub async fn webhook_payment(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> HandlerResult<impl IntoResponse> {
    let order_id = payload["order_id"].as_i64().unwrap_or(-1);
    let is_success = payload["status"]
        .as_str()
        .map(|s| s.to_lowercase() == "success")
        .unwrap_or(false);

    if order_id <= 0 {
        return Err((StatusCode::BAD_REQUEST, "order_id tidak valid".to_string()));
    }

    match Order::process_payment(&state.db, order_id, is_success).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success(&format!(
                        "Webhook: Order {} diperbarui menjadi {}",
                        order_id,
                        if is_success { "PAID" } else { "FAILED" }
                    ))),
                ))
            } else {
                Err((StatusCode::NOT_FOUND, "Order tidak ditemukan.".to_string()))
            }
        }
        Err(e) => Err(internal_server_error(e)),
    }
}