use crate::AppState;
use crate::dtos::auth::{RegisterRequest, RegisterResponse, UserData};
use crate::models::user::User;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use bcrypt::{DEFAULT_COST, hash};
use std::sync::Arc;

/// Register a new user
///
/// Creates a new user account with name, email, and password.
/// Password is hashed using bcrypt before storing.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    )
)]
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
