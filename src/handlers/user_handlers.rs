use axum::{
    extract::{State, Json},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use crate::middleware::auth::AuthUser;
use crate::{AppState, utils::ApiResponse};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    AuthUser { email, .. }: AuthUser, // ✅ pakai curly braces
) -> impl IntoResponse {
    let result = sqlx::query_as!(
        UserProfile,
        "SELECT id, name, email, role FROM users WHERE email = ?",
        email
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(profile) => Json(ApiResponse::success_data("User profile fetched", profile)).into_response(),
        Err(_) => Json(ApiResponse::<()> { 
            status: "error".to_string(), 
            message: "Failed to fetch profile".to_string(), 
            data: None 
        }).into_response(),
    }
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    AuthUser { email, .. }: AuthUser, // ✅ sama juga
    Json(payload): Json<UpdateProfile>,
) -> impl IntoResponse {
    let _ = sqlx::query!(
        "UPDATE users 
         SET name = COALESCE(?, name), 
             email = COALESCE(?, email), 
             password = COALESCE(?, password)
         WHERE email = ?",
        payload.name,
        payload.email,
        payload.password,
        email
    )
    .execute(&state.db)
    .await;

    Json(ApiResponse::<()>::success("Profile updated successfully")).into_response()
}
