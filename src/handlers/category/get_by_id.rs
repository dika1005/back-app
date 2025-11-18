use axum::{ extract::{State, Path}, response::IntoResponse, Json };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::models::category::KategoriModel;

pub async fn get_category_by_id(
    State(state): State<Arc<AppState>>,
    Path(category_id): Path<i32>
) -> impl IntoResponse {
    match KategoriModel::find_by_id(&state.db, category_id).await {
        Ok(Some(category)) => Json(ApiResponse::success_data("Detail kategori berhasil diambil", category)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Kategori tidak ditemukan.".to_string(),
                data: None,
            }),
        ).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()> {
                status: "error".to_string(),
                message: format!("Gagal mengambil detail kategori: {}", e),
                data: None,
            }),
        ).into_response(),
    }
}