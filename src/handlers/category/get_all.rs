use crate::AppState;
use crate::models::category::KategoriModel;
use crate::utils::ApiResponse;
use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;

/// Get all categories
///
/// Returns a list of all product categories.
#[utoipa::path(
    get,
    path = "/categories",
    tag = "categories",
    responses(
        (status = 200, description = "List of categories retrieved successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_all_categories(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match KategoriModel::find_all(&state.db).await {
        Ok(kategori_list) => Json(ApiResponse::success_data(
            "Daftar kategori berhasil diambil",
            kategori_list,
        ))
        .into_response(),
        Err(e) => {
            eprintln!("Error fetching categories: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil data kategori: {}", e),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}
