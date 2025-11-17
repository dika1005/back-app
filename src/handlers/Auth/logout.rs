use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::cookie::{CookieJar, Cookie};
use std::sync::Arc;
use time::Duration;
use chrono::Utc;
use sqlx::Row;
use crate::AppState;
use crate::utils::jwt::verify_jwt;
use serde_json::json;

pub async fn logout_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get("jwt") {
        let token = cookie.value().to_string();
        if let Ok(claims) = verify_jwt(&token) {
            let email = claims.sub;
            if let Ok(Some(row)) = sqlx::query("SELECT id FROM users WHERE email = ?")
                .bind(&email)
                .fetch_optional(&state.db)
                .await
            {
                if let Ok(user_id) = row.try_get::<i64, _>("id") {
                    if let Err(e) = sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE user_id = ?")
                        .bind(user_id)
                        .execute(&state.db)
                        .await
                    {
                        eprintln!("Failed to revoke refresh tokens for user {}: {:?}", user_id, e);
                    }
                }
            }
        }
    }

    let cookie = Cookie::build(("jwt", ""))
        .http_only(true)
        .path("/")
        .secure(std::env::var("SECURE_COOKIE").unwrap_or_else(|_| "false".into()) == "true")
        .max_age(Duration::seconds(0))
        .build();

    let refresh_del = Cookie::build(("refresh_token", ""))
        .http_only(true)
        .path("/")
        .secure(std::env::var("SECURE_COOKIE").unwrap_or_else(|_| "false".into()) == "true")
        .max_age(Duration::seconds(0))
        .build();

    let jar = jar.add(cookie).add(refresh_del);

    (jar, Json(json!({
        "status": "success",
        "message": "Logout berhasil!"
    })))
}