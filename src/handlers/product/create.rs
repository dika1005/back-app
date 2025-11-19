use crate::AppState;
use crate::dtos::product::NewRodProductDto;
use crate::dtos::product::RodProduct;
use crate::middleware::auth::AdminAuth;
use crate::utils::ApiResponse;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn create_product(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Json(new_product_dto): Json<NewRodProductDto>,
) -> HandlerResult<impl IntoResponse> {
    match RodProduct::insert(&state.db, new_product_dto).await {
        Ok(id) => Ok((
            StatusCode::CREATED,
            Json(ApiResponse::success_data_with_message(
                format!("Produk berhasil dibuat dengan ID: {}", id),
                id,
            )),
        )),
        Err(e) => {
            eprintln!("Error creating product: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                "Gagal membuat produk. Pastikan category_id valid.".to_string(),
            ))
        }
    }
}
