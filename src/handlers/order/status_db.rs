use crate::{AppState, dtos::order::Order, utils::ApiResponse};
use axum::{
    Json, // pastikan import Json
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
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

pub async fn get_order_status_db(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<i64>,
) -> HandlerResult<impl IntoResponse> {
    match Order::find_status_by_id(&state.db, order_id).await {
        Ok(status) => Ok((
            StatusCode::OK,
            Json(ApiResponse::success_data_with_message(
                format!("Status Order {} dari database lokal:", order_id).to_string(),
                json!({ "order_id": order_id, "local_status": status }),
            )),
        )),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            format!("Order ID {} tidak ditemukan.", order_id),
        )),
        Err(e) => Err(internal_server_error(e)),
    }
}
