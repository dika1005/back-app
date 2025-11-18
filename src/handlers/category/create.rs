use axum::{ extract::{State, Json}, response::IntoResponse, http::StatusCode };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::models::category::KategoriModel;
use crate::dtos::category::NewKategoriDto;
use crate::middleware::auth::AdminAuth;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Json(new_kategori_dto): Json<NewKategoriDto>
) -> HandlerResult<impl IntoResponse> {
    match KategoriModel::insert(&state.db, new_kategori_dto).await {
        Ok(id) => Ok((
            StatusCode::CREATED,
            Json(ApiResponse::success_data_with_message(format!("Kategori berhasil dibuat dengan ID: {}", id), id)),
        ).into_response()),
        Err(e) => {
            eprintln!("Error creating category: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal membuat kategori: {}", e)))
        }
    }
}