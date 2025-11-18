use axum::{
    extract::{State, Path, Json},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use crate::{
    AppState,
    utils::ApiResponse,
    dtos::order::Order,
    middleware::auth::AdminAuth,
};

type HandlerResult<T> = Result<T, (StatusCode, String)>;

fn internal_server_error(e: sqlx::Error) -> (StatusCode, String) {
    eprintln!("Database Error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan internal pada server.".to_string())
}

pub async fn process_payment(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Path(order_id): Path<i64>,
    Json(payload): Json<serde_json::Value>
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
                    Json(ApiResponse::<()>::success(&format!("Status Order {} diperbarui menjadi {}", order_id, status_msg))),
                ))
            } else {
                Err((StatusCode::NOT_FOUND, "Order tidak ditemukan atau sudah diproses.".to_string()))
            }
        }
        Err(e) => Err(internal_server_error(e)),
    }
}