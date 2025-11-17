use axum::{extract::{State, Query}, response::IntoResponse, Json, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use oauth2::{basic::BasicClient, reqwest::async_http_client, AuthUrl, TokenUrl, ClientId, ClientSecret, RedirectUrl, AuthorizationCode, CsrfToken, Scope};
use oauth2::TokenResponse as _OAuthTokenResponse;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use crate::AppState;
use crate::models::user::User;
use crate::dtos::auth::LoginResponse;
use crate::dtos::auth::UserLoginData;
use crate::utils::jwt::create_jwt;
use axum_extra::extract::cookie::Cookie;
use time::Duration;
use serde_json::json;

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