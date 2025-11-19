use crate::AppState;
use crate::dtos::auth::UpdateRoleRequest;
use crate::models::user::User;
use crate::utils::jwt::verify_jwt;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;
use std::sync::Arc;

pub async fn update_role_handler(
    State(state): State<Arc<AppState>>,
    Path(email): Path<String>,
    jar: CookieJar,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let token = match jar.get("jwt") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err((StatusCode::UNAUTHORIZED, "Token tidak ditemukan".into())),
    };

    let claims = verify_jwt(&token).map_err(|(s, m)| (s, m))?;

    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Kamu bukan admin, gak boleh ubah role!".into(),
        ));
    }

    if payload.role != "admin" && payload.role != "user" {
        return Err((StatusCode::BAD_REQUEST, "Role tidak valid".into()));
    }

    User::update_role(&state.db, &email, &payload.role)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": format!("Role {} berhasil diubah menjadi {}", email, payload.role)
        })),
    ))
}
