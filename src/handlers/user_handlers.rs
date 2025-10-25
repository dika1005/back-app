use axum::{ extract::{ State, Json }, response::IntoResponse, http::StatusCode };
use serde::{ Deserialize};
use std::sync::Arc;

use crate::middleware::auth::AuthUser;
use crate::AppState; 
use crate::utils::ApiResponse;
// Import UserProfile (jika dipindahkan ke model), atau gunakan dari model
use crate::models::user::{ User };

// Hapus definisi UserProfile di sini karena sudah dipindahkan ke model

// DTO untuk request update
#[derive(Deserialize)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>, // CATATAN: Ini harus di-hash sebelum dikirim ke model!
}

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    AuthUser { email, .. }: AuthUser
) -> impl IntoResponse {
    // 1. Panggil method dari model
    let result = User::find_profile_by_email(&state.db, &email).await;

    match result {
        Ok(Some(profile)) =>
            Json(ApiResponse::success_data("User profile fetched", profile)).into_response(),
        Ok(None) =>
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: "User not found".to_string(),
                    data: None,
                }),
            ).into_response(),
        Err(e) =>
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Failed to fetch profile: {}", e),
                    data: None,
                }),
            ).into_response(),
    }
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    AuthUser { email, .. }: AuthUser,
    Json(payload): Json<UpdateProfile>
) -> impl IntoResponse {
    // NOTE PENTING: Password harus di-hash di handler sebelum dikirim ke model!
    let mut hashed_password: Option<String> = None;
    if let Some(ref password) = payload.password {
        // Hash password menggunakan bcrypt
        match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
            Ok(hash) => {
                hashed_password = Some(hash);
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()> {
                        status: "error".to_string(),
                        message: format!("Failed to hash password: {}", e),
                        data: None,
                    }),
                ).into_response();
            }
        }
    }

    // 1. Panggil method dari model dengan data yang sudah diproses (hashed)
    let result = User::update_profile_data(
        &state.db,
        &email,
        &payload.name,
        &payload.email,
        &hashed_password
    ).await;

    match result {
        Ok(_) => Json(ApiResponse::<()>::success("Profile updated successfully")).into_response(),
        Err(e) =>
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()> {
                    status: "error".to_string(),
                    message: format!("Failed to update profile: {}", e),
                    data: None,
                }),
            ).into_response(),
    }
}
