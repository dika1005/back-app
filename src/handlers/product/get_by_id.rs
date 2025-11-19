use crate::AppState;
use crate::dtos::product::RodProduct;
use crate::utils::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

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
