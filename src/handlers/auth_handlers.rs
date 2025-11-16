use axum::{extract::{State, Query, Path}, response::IntoResponse, http::StatusCode, Json};
use bcrypt::{verify, hash, DEFAULT_COST};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use time::Duration;
use chrono::{Utc, Duration as ChronoDuration};
use sqlx::Row;

use oauth2::{
    basic::BasicClient,
    reqwest::async_http_client,
    AuthUrl,
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    RedirectUrl,
    Scope,
    TokenUrl,
};

// bring oauth2 TokenResponse trait into scope for `access_token()` helper
use oauth2::TokenResponse as _OAuthTokenResponse;

use crate::AppState;
use crate::models::user::User;
use crate::dtos::auth::{
    RegisterRequest,
    RegisterResponse,
    UserData,
    LoginRequest,
    LoginResponse,
    UserLoginData,
    UpdateRoleRequest,
};

// JWT utils
use crate::utils::jwt::{create_jwt, create_refresh_token, verify_jwt, verify_refresh_token};

// ======================================
// REGISTER HANDLER
// ======================================
pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, String)> {
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

// ======================================
// LOGIN HANDLER
// ======================================
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

    // set refresh cookie on root path so clients reliably send it to /auth/refresh
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

// ======================================
// LOGOUT HANDLER
// ======================================
pub async fn logout_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    // If a jwt cookie exists, try to revoke refresh tokens for that user in DB.
    if let Some(cookie) = jar.get("jwt") {
        let token = cookie.value().to_string();
        if let Ok(claims) = verify_jwt(&token) {
            // claims.sub for login token is email (see login_handler)
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

// ======================================
// GOOGLE AUTH & CALLBACK
// ======================================
pub async fn google_auth_handler() -> impl IntoResponse {
    let client_id = ClientId::new(std::env::var("GOOGLE_CLIENT_ID").unwrap());
    let client_secret = ClientSecret::new(std::env::var("GOOGLE_CLIENT_SECRET").unwrap());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap();
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap();

    let redirect_url = RedirectUrl::new("http://localhost:3001/auth/google/callback".into()).unwrap();

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .url();

    (StatusCode::FOUND, [(axum::http::header::LOCATION, auth_url.to_string())])
}

pub async fn google_callback_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let code = match params.get("code") {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "error", "message": "Kode otorisasi tidak ditemukan"})),
            )
                .into_response();
        }
    };

    let client_id = ClientId::new(std::env::var("GOOGLE_CLIENT_ID").unwrap());
    let client_secret = ClientSecret::new(std::env::var("GOOGLE_CLIENT_SECRET").unwrap());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap();
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap();
    let redirect_url = RedirectUrl::new("http://localhost:3001/auth/google/callback".into()).unwrap();

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url);

    let token_result = match client.exchange_code(AuthorizationCode::new(code)).request_async(async_http_client).await {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error exchange token: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": "Gagal mengambil token"})),
            )
                .into_response();
        }
    };

    let access_token = token_result.access_token().secret();

    let user_info = match reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(res) => match res.json::<Value>().await {
            Ok(data) => data,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(json!({"status":"error","message":"Gagal parsing data user"}))).into_response();
            }
        },
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({"status":"error","message":"Gagal mengambil data user"}))).into_response();
        }
    };

    let email = user_info["email"].as_str().unwrap_or("").to_string();
    let name = user_info["name"].as_str().unwrap_or("Pengguna Google").to_string();

    if let Err(e) = User::upsert_google_user(&state.db, &email, &name).await {
        eprintln!("Error saat upsert Google user: {:?}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status":"error","message":"Gagal menyimpan data pengguna"}))).into_response();
    }

    let token = create_jwt(email.clone(), "user".to_string(), 5).expect("Gagal membuat token JWT untuk Google Auth");

    let cookie = Cookie::build(("jwt", token.clone()))
        .http_only(true)
        .secure(std::env::var("SECURE_COOKIE").unwrap_or_else(|_| "false".into()) == "true")
        .path("/")
        .max_age(Duration::hours(2))
        .build();

    let updated_jar = jar.add(cookie);

    let user_data = UserLoginData { email: email.clone(), role: "user".to_string() };

    (updated_jar, Json(LoginResponse {
        status: "success".into(),
        message: "Login berhasil".into(),
        token: Some(token),
        user: Some(user_data),
    })).into_response()
}

// ======================================
// UPDATE ROLE HANDLER
// ======================================
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
        return Err((StatusCode::FORBIDDEN, "Kamu bukan admin, gak boleh ubah role!".into()));
    }

    if payload.role != "admin" && payload.role != "user" {
        return Err((StatusCode::BAD_REQUEST, "Role tidak valid".into()));
    }

    User::update_role(&state.db, &email, &payload.role)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, Json(json!({
        "status": "success",
        "message": format!("Role {} berhasil diubah menjadi {}", email, payload.role)
    }))))
}

// ======================================
// REFRESH TOKEN HANDLER
// ======================================
#[derive(serde::Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(serde::Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    payload: Option<Json<RefreshRequest>>,
) -> Result<(StatusCode, Json<TokenResponse>), (StatusCode, String)> {
    // Determine refresh token: prefer JSON body, otherwise read cookie `refresh_token`
    let refresh_token = if let Some(Json(json)) = payload {
        json.refresh_token
    } else if let Some(cookie) = jar.get("refresh_token") {
        cookie.value().to_string()
    } else {
        return Err((StatusCode::BAD_REQUEST, "refresh_token not provided".into()));
    };

    // verify refresh token structure and signature
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

    // rotate: revoke old token, insert new refresh token and create access token
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
