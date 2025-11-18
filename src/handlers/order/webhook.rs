use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
// removed unused import: serde_json::json
use crate::{
    AppState,
    utils::ApiResponse,
    dtos::order::Order,
};

type HandlerResult<T> = Result<T, (StatusCode, String)>;

fn internal_server_error(e: sqlx::Error) -> (StatusCode, String) {
    eprintln!("Database Error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan internal pada server.".to_string())
}

pub async fn webhook_payment(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>
) -> HandlerResult<impl IntoResponse> {
    let order_id = match payload["order_id"].as_str() {
        Some(s) => s.parse::<i64>().unwrap_or(-1),
        None => payload["order_id"].as_i64().unwrap_or(-1),
    };

    let transaction_status = payload["transaction_status"].as_str().unwrap_or("");
    let is_success = transaction_status == "settlement" || transaction_status == "capture";

    if order_id <= 0 {
        return Err((StatusCode::BAD_REQUEST, "order_id tidak valid".to_string()));
    }

    eprintln!("[Webhook] Menerima notifikasi untuk Order ID: {}", order_id);
    eprintln!("[Webhook] Status Transaksi Midtrans: {}", transaction_status);

    match Order::process_payment(&state.db, order_id, is_success).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success(&format!("Webhook: Order {} diperbarui menjadi {}", order_id, if is_success { "PAID" } else { "FAILED" }))),
                ))
            } else {
                eprintln!("[Webhook] Order {} tidak ditemukan atau sudah diproses (Status tidak diubah). Mengembalikan 200 OK.", order_id);
                Ok((StatusCode::OK, Json(ApiResponse::<()>::success("Order sudah diproses."))))
            }
        }
        Err(e) => Err(internal_server_error(e)),
    }
}