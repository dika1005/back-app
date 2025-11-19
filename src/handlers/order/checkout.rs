use crate::{
    AppState, dtos::order::NewOrderDto, middleware::auth::AuthUser, models::user::User,
    utils::ApiResponse,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64::Engine;
use base64::engine::general_purpose;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

fn internal_server_error(e: sqlx::Error) -> (StatusCode, String) {
    eprintln!("Database Error: {}", e);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Terjadi kesalahan internal pada server.".to_string(),
    )
}

pub async fn checkout(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(new_order_dto): Json<NewOrderDto>,
) -> HandlerResult<impl IntoResponse> {
    let user_record = match User::find_by_email(&state.db, &auth_user.email).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err((StatusCode::UNAUTHORIZED, "User tidak ditemukan".to_string())),
        Err(e) => return Err(internal_server_error(e)),
    };

    match crate::dtos::order::Order::create_order(&state.db, user_record.id, new_order_dto).await {
        Ok(order_id) => {
            let client = Client::new();

            let payload = json!({
                "transaction_details": {
                    "order_id": order_id.to_string(),
                    "gross_amount": 100000
                },
                "customer_details": {
                    "first_name": user_record.name,
                    "email": user_record.email
                },
                "credit_card": { "secure": true }
            });

            let url = format!("{}/snap/v1/transactions", state.midtrans_base_url);
            let server_key = &state.midtrans_server_key;
            let auth_string = format!("{}:", server_key);
            let encoded_auth = general_purpose::STANDARD.encode(auth_string);

            let masked_key = if server_key.len() > 8 {
                format!("{}*****", &server_key[..8])
            } else {
                "*****".to_string()
            };
            eprintln!("[midtrans] URL: {}", url);
            eprintln!("[midtrans] Server key (masked): {}", masked_key);
            eprintln!(
                "[midtrans] Auth Header (Partial): Basic {}...",
                &encoded_auth[..15]
            );

            let resp = client
                .post(&url)
                .header("Authorization", format!("Basic {}", encoded_auth))
                .json(&payload)
                .send()
                .await;

            match resp {
                Ok(r) => {
                    let status = r.status();

                    if status.is_success() {
                        let data: serde_json::Value = r.json().await.unwrap_or_default();
                        let redirect_url = data["redirect_url"].as_str().unwrap_or("").to_string();

                        Ok((
                            StatusCode::CREATED,
                            Json(ApiResponse::success_data_with_message(
                                "Checkout berhasil. Silakan lanjutkan pembayaran melalui Midtrans."
                                    .to_string(),
                                json!({ "order_id": order_id, "payment_url": redirect_url }),
                            )),
                        ))
                    } else {
                        let error_body = r
                            .text()
                            .await
                            .unwrap_or_else(|_| "Failed to read response body".to_string());
                        eprintln!("Midtrans Error Status: {}", status);
                        eprintln!("Midtrans Error Body: {}", error_body);
                        Err((
                            StatusCode::BAD_GATEWAY,
                            format!(
                                "Gagal membuat transaksi di Midtrans. Detail: {}",
                                error_body
                            ),
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
