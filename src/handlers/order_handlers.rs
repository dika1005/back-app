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

            // Debug: tampilkan URL dan key (dengan masking)
            let server_key = &state.midtrans_server_key;
            let masked_key = if server_key.len() > 8 {
                format!("{}*****", &server_key[..8])
            } else {
                "*****".to_string()
            };
            eprintln!("[midtrans] URL: {}", url);
            eprintln!("[midtrans] Server key (masked): {}", masked_key);

            // 🔐 Request ke Midtrans Snap API
            let resp = client
                .post(&url)
                .basic_auth(&state.midtrans_server_key, Some("")) // pakai Server Key, bukan Client
                .json(&payload)
                .send()
                .await;

            match resp {
                Ok(r) => {
                    if r.status().is_success() {
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
                        eprintln!("Midtrans Error: {:?}", r.text().await);
                        Err((
                            StatusCode::BAD_GATEWAY,
                            "Gagal membuat transaksi di Midtrans.".to_string(),
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
