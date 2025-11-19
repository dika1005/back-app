use crate::AppState;
use crate::dtos::category::NewKategoriDto;
use crate::middleware::auth::AdminAuth;
use crate::models::category::KategoriModel;
use crate::utils::ApiResponse;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Json(new_kategori_dto): Json<NewKategoriDto>,
) -> HandlerResult<impl IntoResponse> {
    match KategoriModel::insert(&state.db, new_kategori_dto).await {
        Ok(id) => Ok((
            StatusCode::CREATED,
            Json(ApiResponse::success_data_with_message(
                format!("Kategori berhasil dibuat dengan ID: {}", id),
                id,
            )),
        )
            .into_response()),
        Err(e) => {
            eprintln!("Error creating category: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Gagal membuat kategori: {}", e),
            ))
        }
    }
}
