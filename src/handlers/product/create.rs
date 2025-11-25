use crate::AppState;
use crate::dtos::product::NewRodProductDto;
use crate::dtos::product::RodProduct;
use crate::middleware::auth::AdminAuth;
use crate::utils::ApiResponse;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

type HandlerResult<T> = Result<T, (StatusCode, String)>;

/// Create a new product (Admin only)
///
/// Creates a new fishing rod product in the catalog.
/// Requires admin authentication.
#[utoipa::path(
    post,
    path = "/products/create",
    tag = "products",
    request_body = NewRodProductDto,
    responses(
        (status = 201, description = "Product created successfully"),
        (status = 400, description = "Invalid input or category_id"),
        (status = 401, description = "Unauthorized - Admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_product(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
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
