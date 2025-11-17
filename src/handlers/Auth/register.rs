use axum::{extract::State, response::IntoResponse, Json, http::StatusCode};
use bcrypt::{hash, DEFAULT_COST};
use std::sync::Arc;
use crate::AppState;
use crate::models::user::User;
use crate::dtos::auth::{RegisterRequest, RegisterResponse, UserData};

pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let is_registered = User::exists_by_email(&state.db, &payload.email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if is_registered {
        return Err((StatusCode::BAD_REQUEST, "Email sudah terdaftar".into()));
    }

    let hashed = hash(&payload.password, DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    User::insert(
        &state.db,
        &payload.name,
        &payload.email,
        &hashed,
        payload.alamat.as_ref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            status: "success".into(),
            message: "Registrasi berhasil!".into(),
            user: Some(UserData {
                name: payload.name,
                email: payload.email,
                alamat: payload.alamat,
            }),
        }),
    ))
}