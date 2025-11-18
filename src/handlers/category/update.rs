use axum::{ extract::{State, Path, Json}, response::IntoResponse, http::StatusCode };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::models::category::KategoriModel;
use crate::dtos::category::NewKategoriDto;
use crate::middleware::auth::AdminAuth;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn update_category(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Path(category_id): Path<i32>,
    Json(updated_kategori_dto): Json<NewKategoriDto>
) -> HandlerResult<impl IntoResponse> {
    match KategoriModel::update(&state.db, category_id, &updated_kategori_dto.name).await {
        Ok(_) => Ok((StatusCode::OK, Json(ApiResponse::<()>::success("Kategori berhasil diperbarui"))).into_response()),
        Err(e) => {
            eprintln!("Error updating category: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal memperbarui kategori: {}", e)))
        }
    }
}