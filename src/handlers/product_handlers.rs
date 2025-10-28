// src/handlers/product_handlers.rs

use axum::{ extract::{ Path, State, Json }, response::IntoResponse, http::StatusCode };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::dtos::product::{ NewRodProductDto, RodProduct };
use crate::middleware::auth::AdminAuth; // 1. TAMBAH GUARD
// Hapus: use crate::middleware::auth::AuthUser; (Tidak diperlukan karena AdminAuth mengurusnya)

// Type alias untuk Result yang konsisten dengan rejection AdminAuth
type HandlerResult<T> = Result<T, (StatusCode, String)>;

// --- 1. POST /products (CREATE) - GUARDED BY ADMINAUTH ---
pub async fn create_product(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth, // <-- RBAC GUARD
    Json(new_product_dto): Json<NewRodProductDto>
) -> HandlerResult<impl IntoResponse> {
    match RodProduct::insert(&state.db, new_product_dto).await {
        Ok(id) => {
            Ok((
                StatusCode::CREATED,
                Json(
                    ApiResponse::success_data_with_message(
                        format!("Produk berhasil dibuat dengan ID: {}", id),
                        id
                    )
                ),
            ))
        }
        Err(e) => {
            eprintln!("Error creating product: {}", e);
            // Rejection manual: 400 Bad Request
            Err((
                StatusCode::BAD_REQUEST,
                "Gagal membuat produk. Pastikan category_id valid.".to_string(),
            ))
        }
    }
}

// --- 2. GET /products (READ ALL) - PUBLIC ---
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
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil daftar produk: {}", e),
                    data: None,
                }),
            ).into_response()
        }
    }
}

// --- 3. GET /products/:id (READ DETAIL) - PUBLIC ---
pub async fn find_product_by_id(
    State(state): State<Arc<AppState>>,
    Path(product_id): Path<i64>
) -> impl IntoResponse {
    match RodProduct::find_detail_by_id(&state.db, product_id).await {
        Ok(Some(product_detail)) => {
            Json(
                ApiResponse::success_data("Detail produk berhasil diambil", product_detail)
            ).into_response()
        }
        Ok(None) => {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: "Produk tidak ditemukan.".to_string(),
                    data: None,
                }),
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil detail produk: {}", e),
                    data: None,
                }),
            ).into_response()
        }
    }
}

// --- 4. PUT /products/:id (UPDATE) - GUARDED BY ADMINAUTH ---
pub async fn update_product(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth, // <-- RBAC GUARD
    Path(product_id): Path<i64>,
    Json(updated_product_dto): Json<NewRodProductDto>
) -> HandlerResult<impl IntoResponse> {
    match RodProduct::update(&state.db, product_id, updated_product_dto).await {
        Ok(_) => {
            Ok((StatusCode::OK, Json(ApiResponse::<()>::success("Produk berhasil diperbarui"))))
        }
        Err(e) => {
            eprintln!("Error updating product: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal memperbarui produk: {}", e)))
        }
    }
}

// --- 5. DELETE /products/:id (DELETE) - GUARDED BY ADMINAUTH ---
pub async fn delete_product(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth, // <-- RBAC GUARD
    Path(product_id): Path<i64>
) -> HandlerResult<impl IntoResponse> {
    match RodProduct::delete(&state.db, product_id).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok((StatusCode::OK, Json(ApiResponse::<()>::success("Produk berhasil dihapus"))))
            } else {
                Err((StatusCode::NOT_FOUND, "Produk tidak ditemukan.".to_string()))
            }
        }
        Err(e) => {
            eprintln!("Error deleting product: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal menghapus produk: {}", e)))
        }
    }
}
