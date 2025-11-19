use crate::{AppState, middleware::auth::AuthUser, utils::ApiResponse};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64::Engine;
use base64::engine::general_purpose;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn query_midtrans_status(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Path(order_id): Path<i64>,
) -> HandlerResult<impl IntoResponse> {
    let client = Client::new();
    let url = format!("{}/v2/{}/status", state.midtrans_base_url, order_id);
    let server_key = &state.midtrans_server_key;
    let auth_string = format!("{}:", server_key);
    let encoded_auth = general_purpose::STANDARD.encode(auth_string);

    eprintln!(
        "[Midtrans Query] Querying status for Order ID: {} at URL: {}",
        order_id, url
    );

    let resp = client
        .get(&url)
        .header("Authorization", format!("Basic {}", encoded_auth))
        .send()
        .await;

    match resp {
        Ok(r) => {
            let status = r.status();
            let data: serde_json::Value = r.json().await.unwrap_or_default();

            if status.is_success() {
                let transaction_status = data["transaction_status"].as_str().unwrap_or("UNKNOWN");

                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::success_data_with_message(
                        format!("Status Midtrans untuk Order {}:", order_id).to_string(),
                        json!({
                            "order_id": order_id,
                            "midtrans_status": transaction_status,
                            "full_response": data
                        }),
                    )),
                ))
            } else {
                eprintln!(
                    "[Midtrans Query Error] Status: {}, Body: {:?}",
                    status, data
                );
                let error_msg = data["status_message"]
                    .as_str()
                    .unwrap_or("Gagal Query Status");
                Err((
                    StatusCode::BAD_GATEWAY,
                    format!("Gagal menghubungi Midtrans: {}", error_msg),
                ))
            }
        }
        Err(e) => {
            eprintln!("[Midtrans Query Request Error]: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                "Tidak dapat terhubung ke Midtrans untuk cek status.".to_string(),
            ))
        }
    }
}
