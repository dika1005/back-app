use crate::AppState;
use crate::dtos::product::RodProduct;
use crate::middleware::auth::AdminAuth;
use crate::utils::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

pub async fn delete_product(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth,
    Path(product_id): Path<i64>,
) -> HandlerResult<impl IntoResponse> {
    match RodProduct::delete(&state.db, product_id).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success("Produk berhasil dihapus")),
                ))
            } else {
                Err((StatusCode::NOT_FOUND, "Produk tidak ditemukan.".to_string()))
            }
        }
        Err(e) => {
            eprintln!("Error deleting product: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Gagal menghapus produk: {}", e),
            ))
        }
    }
}
