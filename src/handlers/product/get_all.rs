use axum::{ extract::State, response::IntoResponse, Json };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::dtos::product::RodProduct;

pub async fn get_all_products(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match RodProduct::find_all_details(&state.db).await {
        Ok(product_list) => {
            Json(
                ApiResponse::success_data("Daftar produk berhasil diambil", product_list)
            ).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching products: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil daftar produk: {}", e),
                    data: None,
                }),
            ).into_response()
        }
    }
}
