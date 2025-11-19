use axum::{ extract::{State, Path, Json}, response::IntoResponse, http::StatusCode };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::dtos::product::{ NewRodProductDto, RodProduct };
use crate::middleware::auth::AdminAuth;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn update_product(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Path(product_id): Path<i64>,
    Json(updated_product_dto): Json<NewRodProductDto>,
) -> HandlerResult<impl IntoResponse> {
    match RodProduct::update(&state.db, product_id, updated_product_dto).await {
        Ok(_) => Ok((StatusCode::OK, Json(ApiResponse::<()>::success("Produk berhasil diperbarui")))),
        Err(e) => {
            eprintln!("Error updating product: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal memperbarui produk: {}", e)))
        }
    }
}
