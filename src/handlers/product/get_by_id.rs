use crate::AppState;
use crate::dtos::product::RodProduct;
use crate::utils::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

/// Get product by ID
///
/// Returns detailed information about a specific fishing rod product.
#[utoipa::path(
    get,
    path = "/products/{id}",
    tag = "products",
    params(
        ("id" = i64, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product details retrieved successfully"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn find_product_by_id(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<i64>,
) -> impl IntoResponse {
    match RodProduct::find_detail_by_id(&state.db, product_id).await {
        Ok(Some(product_detail)) => Json(ApiResponse::success_data(
            "Detail produk berhasil diambil",
            product_detail,
        ))
        .into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Produk tidak ditemukan.".to_string(),
                data: None,
            }),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()> {
                status: "error".to_string(),
                message: format!("Gagal mengambil detail produk: {}", e),
                data: None,
            }),
        )
            .into_response(),
    }
}
