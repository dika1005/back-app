use axum::{ extract::State, response::IntoResponse, Json };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::models::category::KategoriModel;

pub async fn get_all_categories(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match KategoriModel::find_all(&state.db).await {
        Ok(kategori_list) => {
            Json(ApiResponse::success_data("Daftar kategori berhasil diambil", kategori_list)).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching categories: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil data kategori: {}", e),
                    data: None,
                }),
            ).into_response()
        }
    }
}