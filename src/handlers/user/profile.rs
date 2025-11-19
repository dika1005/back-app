use axum::{ extract::{ State, Json }, response::IntoResponse, http::StatusCode };
use std::sync::Arc;

use crate::middleware::auth::AuthUser;
use crate::AppState;
use crate::utils::ApiResponse;
use crate::models::user::User;
use crate::dtos::user::UpdateProfile;

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    AuthUser { email, .. }: AuthUser
) -> impl IntoResponse {
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
    let mut hashed_password: Option<String> = None;
    if let Some(ref password) = payload.password {
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
