use axum::{extract::State, Json, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;
use crate::AppState;
use crate::utils::jwt::{verify_refresh_token, create_jwt, create_refresh_token};
use sqlx::Row;
use chrono::{Utc, Duration as ChronoDuration};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    payload: Option<Json<RefreshRequest>>,
) -> Result<(StatusCode, Json<TokenResponse>), (StatusCode, String)> {
    let refresh_token = if let Some(Json(json)) = payload {
        json.refresh_token
    } else if let Some(cookie) = jar.get("refresh_token") {
        cookie.value().to_string()
    } else {
        return Err((StatusCode::BAD_REQUEST, "refresh_token not provided".into()));
    };

    let claims = verify_refresh_token(&refresh_token).map_err(|e| e)?;

    let user_id: i64 = claims.sub.parse().map_err(|_| (StatusCode::UNAUTHORIZED, "invalid subject in token".into()))?;

    let row = sqlx::query("SELECT id, revoked, expires_at FROM refresh_tokens WHERE token = ?")
        .bind(&refresh_token)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rec = match row {
        Some(r) => r,
        None => return Err((StatusCode::UNAUTHORIZED, "refresh token not found".into())),
    };

    let revoked: bool = rec.try_get("revoked").map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let expires_at: chrono::NaiveDateTime = rec.try_get("expires_at").map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if revoked {
        return Err((StatusCode::UNAUTHORIZED, "refresh token revoked".into()));
    }
    if Utc::now().naive_utc() > expires_at {
        return Err((StatusCode::UNAUTHORIZED, "refresh token expired".into()));
    }

    let mut tx = state.db.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE token = ?")
        .bind(&refresh_token)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user_row = sqlx::query("SELECT role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let role: String = match user_row {
        Some(r) => r.try_get("role").unwrap_or_else(|_| "user".to_string()),
        None => return Err((StatusCode::NOT_FOUND, "user not found".into())),
    };

    let access = create_jwt(user_id.to_string(), role.clone(), 5).map_err(|e| e)?;
    let refresh = create_refresh_token(user_id.to_string(), 5).map_err(|e| e)?;
    let new_expires_at = Utc::now().naive_utc() + ChronoDuration::days(5);

    sqlx::query("INSERT INTO refresh_tokens (user_id, token, revoked, expires_at) VALUES (?, ?, false, ?)")
        .bind(user_id)
        .bind(&refresh)
        .bind(new_expires_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let resp = TokenResponse { access_token: access, refresh_token: refresh };
    Ok((StatusCode::OK, Json(resp)))
}