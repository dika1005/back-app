use axum::{extract::State, response::IntoResponse, Json, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use bcrypt::verify;
use time::Duration;
use chrono::{Utc, Duration as ChronoDuration};
use std::sync::Arc;
use crate::AppState;
use crate::models::user::User;
use crate::dtos::auth::{LoginRequest, LoginResponse, UserLoginData};
use crate::utils::jwt::{create_jwt, create_refresh_token};
use sqlx::Row;
use serde_json::json;
use axum_extra::extract::cookie::Cookie;

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_option = User::find_by_email(&state.db, &payload.email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = match user_option {
        Some(u) => u,
        None => return Err((StatusCode::UNAUTHORIZED, "Email atau password salah".into())),
    };

    let stored_password_hash: String = user.password;
    let role: String = user.role;
    let user_id: i64 = user.id;

    if !verify(&payload.password, &stored_password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Verifikasi gagal".into()))?
    {
        return Err((StatusCode::UNAUTHORIZED, "Email atau password salah".into()));
    }

    // access token 5 menit
    let access_token = create_jwt(payload.email.clone(), role.clone(), 5).map_err(|e| e)?;

    // refresh token (5 hari)
    let refresh_token = create_refresh_token(user_id.to_string(), 5).map_err(|e| e)?;

    let expires_at = Utc::now().naive_utc() + ChronoDuration::days(5);
    sqlx::query("INSERT INTO refresh_tokens (user_id, token, revoked, expires_at) VALUES (?, ?, false, ?)")
        .bind(user_id)
        .bind(&refresh_token)
        .bind(expires_at)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let secure_cookie = std::env::var("SECURE_COOKIE").unwrap_or_else(|_| "false".into()) == "true";

    let access_cookie = Cookie::build(("jwt", access_token.clone()))
        .http_only(true)
        .secure(secure_cookie)
        .path("/")
        .max_age(Duration::minutes(5))
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token.clone()))
        .http_only(true)
        .secure(secure_cookie)
        .path("/")
        .max_age(Duration::days(30))
        .build();

    let updated_jar = jar.add(access_cookie).add(refresh_cookie);

    Ok((
        updated_jar,
        Json(LoginResponse {
            status: "success".into(),
            message: "Login berhasil!".into(),
            token: Some(access_token),
            user: Some(UserLoginData {
                email: payload.email,
                role: role.clone(),
            }),
        }),
    ))
}