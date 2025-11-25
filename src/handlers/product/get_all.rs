use crate::AppState;
use crate::dtos::product::RodProduct;
use crate::utils::ApiResponse;
use axum::{Json, extract::{State, Query}, response::IntoResponse};
use std::sync::Arc;
use crate::dtos::pagination::PaginationParams;

/// Get all products with pagination
///
/// Returns a paginated list of all fishing rod products with their details.
#[utoipa::path(
    get,
    path = "/products",
    tag = "products",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of products retrieved successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_all_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    match RodProduct::get_all_paginated(&state.db, params).await {
        Ok(paginated) => Json(ApiResponse::success_data(
            "Daftar produk berhasil diambil",
            paginated,
        ))
        .into_response(),
        Err(e) => {
            eprintln!("Error fetching products: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil daftar produk: {}", e),
                    data: None,
                }),
            )
                .into_response()
        }
    }
}
