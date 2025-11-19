use crate::AppState;
use crate::middleware::auth::AdminAuth;
use crate::models::category::KategoriModel;
use crate::utils::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Path(category_id): Path<i32>,
) -> HandlerResult<impl IntoResponse> {
    match KategoriModel::delete(&state.db, category_id).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success("Kategori berhasil dihapus")),
                )
                    .into_response())
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    "Kategori tidak ditemukan.".to_string(),
                ))
            }
        }
        Err(e) => {
            eprintln!("Error deleting category: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Gagal menghapus kategori: {}", e),
            ))
        }
    }
}
