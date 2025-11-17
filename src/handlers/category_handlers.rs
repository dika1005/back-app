use axum::{ extract::{ Path, State, Json }, response::IntoResponse, http::StatusCode };
use std::sync::Arc;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::models::category::KategoriModel; // <-- IMPORT STRUCT MODEL YANG BENAR
use crate::dtos::category::{ NewKategoriDto };
use crate::middleware::auth::AdminAuth; // <-- IMPORT ADMINAUTH

// Type alias untuk Result yang konsisten dengan rejection AdminAuth
type HandlerResult<T> = Result<T, (StatusCode, String)>;

// --- 1. GET /categories (READ ALL) - PUBLIC ---
pub async fn get_all_categories(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Perbaikan: Panggil KategoriModel::find_all
    match KategoriModel::find_all(&state.db).await {
        Ok(kategori_list) => {
            Json(
                ApiResponse::success_data("Daftar kategori berhasil diambil", kategori_list)
            ).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching categories: {}", e);
            // ... (Error response)
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil data kategori: {}", e),
                    data: None,
                }),
            ).into_response()
        }
    }
}

// --- 2. POST /categories (CREATE) - GUARDED BY ADMINAUTH ---
pub async fn create_category(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth, // <-- TAMBAH GUARD INI
    Json(new_kategori_dto): Json<NewKategoriDto>
) -> HandlerResult<impl IntoResponse> {
    // <-- UBAH RETURN TYPE
    // Perbaikan: Panggil KategoriModel::insert
    match KategoriModel::insert(&state.db, new_kategori_dto).await {
        Ok(id) => {
            Ok(
                (
                    StatusCode::CREATED,
                    Json(
                        ApiResponse::success_data_with_message(
                            format!("Kategori berhasil dibuat dengan ID: {}", id),
                            id
                        )
                    ),
                ).into_response()
            )
        }
        Err(e) => {
            eprintln!("Error creating category: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal membuat kategori: {}", e)))
        }
    }
}

// --- 3. GET /categories/:id (READ DETAIL) - PUBLIC ---
pub async fn get_category_by_id(
    State(state): State<Arc<AppState>>,
    Path(category_id): Path<i32>
) -> impl IntoResponse {
    // Perbaikan: Panggil KategoriModel::find_by_id
    match KategoriModel::find_by_id(&state.db, category_id).await {
        Ok(Some(category)) => {
            Json(
                ApiResponse::success_data("Detail kategori berhasil diambil", category)
            ).into_response()
        }
        Ok(None) => {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: "Kategori tidak ditemukan.".to_string(),
                    data: None,
                }),
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Gagal mengambil detail kategori: {}", e),
                    data: None,
                }),
            ).into_response()
        }
    }
}

// --- 4. PUT /categories/:id (UPDATE) - GUARDED BY ADMINAUTH ---
pub async fn update_category(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth, // <-- TAMBAH GUARD INI
    Path(category_id): Path<i32>,
    Json(updated_kategori_dto): Json<NewKategoriDto>
) -> HandlerResult<impl IntoResponse> {
    // <-- UBAH RETURN TYPE
    // Perbaikan: Panggil KategoriModel::update (dan berikan &str)
    match KategoriModel::update(&state.db, category_id, &updated_kategori_dto.name).await {
        Ok(_) => {
            Ok(
                (
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success("Kategori berhasil diperbarui")),
                ).into_response()
            )
        }
        Err(e) => {
            eprintln!("Error updating category: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal memperbarui kategori: {}", e)))
        }
    }
}

// --- 5. DELETE /categories/:id (DELETE) - GUARDED BY ADMINAUTH ---
pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    AdminAuth(_): AdminAuth, // <-- TAMBAH GUARD INI
    Path(category_id): Path<i32>
) -> HandlerResult<impl IntoResponse> {
    // <-- UBAH RETURN TYPE
    // Perbaikan: Panggil KategoriModel::delete
    match KategoriModel::delete(&state.db, category_id).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(
                    (
                        StatusCode::OK,
                        Json(ApiResponse::<()>::success("Kategori berhasil dihapus")),
                    ).into_response()
                )
            } else {
                Err((StatusCode::NOT_FOUND, "Kategori tidak ditemukan.".to_string()))
            }
        }
        Err(e) => {
            eprintln!("Error deleting category: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal menghapus kategori: {}", e)))
        }
    }
}
