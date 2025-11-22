use crate::AppState;
use crate::dtos::product::RodProduct;
use crate::utils::ApiResponse;
use axum::{Json, extract::{State, Query}, response::IntoResponse};
use std::sync::Arc;
use crate::dtos::pagination::PaginationParams;

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
