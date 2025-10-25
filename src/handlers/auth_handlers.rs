use axum::{ extract::{ State, Query, Path }, response::IntoResponse, http::StatusCode, Json };

use bcrypt::{ verify, hash, DEFAULT_COST };
use serde_json::Value;
use std::{ collections::HashMap, sync::Arc };
 // Tetap simpan jika ada kode yang memerlukannya
use chrono::{ Utc, Duration as ChronoDuration };
use jsonwebtoken::{ encode, Header, EncodingKey };
use axum_extra::extract::cookie::{ Cookie, CookieJar };
use time::Duration;
use jsonwebtoken::Validation;
use serde_json::json;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;

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
    TokenResponse,
    TokenUrl,
};


// Import AppState dan model User
use crate::AppState;
use crate::models::user::User;
use crate::dtos::auth::{
    RegisterRequest, RegisterResponse, UserData,
    LoginRequest, LoginResponse, UserLoginData,
    Claims, UpdateRoleRequest
};

pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, String)> {
    // 1. Cek apakah email sudah terdaftar menggunakan model
    let is_registered = User::exists_by_email(&state.db, &payload.email).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        e.to_string(),
    ))?;

    if is_registered {
        return Err((StatusCode::BAD_REQUEST, "Email sudah terdaftar".into()));
    }

    // 2. Hash password
    let hashed = hash(&payload.password, DEFAULT_COST).map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        e.to_string(),
    ))?;

    // 3. Masukkan ke database menggunakan model
    User::insert(
        &state.db,
        &payload.name,
        &payload.email,
        &hashed,
        payload.alamat.as_ref()
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
    Json(payload): Json<LoginRequest>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Cari user menggunakan model
    let user_option = User::find_by_email(&state.db, &payload.email).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        e.to_string(),
    ))?;

    let user = match user_option {
        Some(u) => u,
        None => {
            return Err((StatusCode::UNAUTHORIZED, "Email atau password salah".into()));
        }
    };

    // Ambil data langsung dari struct User
    let stored_password_hash: String = user.password;
    let role: String = user.role;

    // 2. Verifikasi password
    if
        !verify(&payload.password, &stored_password_hash).map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Verifikasi gagal".into(),
        ))?
    {
        return Err((StatusCode::UNAUTHORIZED, "Email atau password salah".into()));
    }

    // 3. Logika JWT (dibiarkan sama)
    let secret = std::env
        ::var("JWT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET not set".into()))?;

    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::hours(1))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: payload.email.clone(),
        role: role.clone(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cookie = Cookie::build(("jwt", token.clone()))
        .http_only(true)
        .secure(false) // ubah ke true kalau udah HTTPS
        .path("/")
        .max_age(Duration::hours(2))
        .build();

    let updated_jar = jar.add(cookie);

    Ok((
        updated_jar,
        Json(LoginResponse {
            status: "success".into(),
            message: "Login berhasil!".into(),
            token: Some(token),
            user: Some(UserLoginData {
                email: payload.email,
                role: role.clone(),
            }),
        }),
    ))
}

// ======================================
// LOGOUT HANDLER (Dibiarkan sama)
// ======================================
pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("jwt", ""))
        .http_only(true)
        .path("/")
        .max_age(Duration::seconds(0))
        .build();

    let jar = jar.add(cookie);

    (jar, Json(serde_json::json!({
  "status": "success",
  "message": "Logout berhasil!"
 })))
}

// ======================================
// GOOGLE AUTH HANDLER (Dibiarkan sama)
// ======================================
pub async fn google_auth_handler() -> impl IntoResponse {
    let client_id = ClientId::new(std::env::var("GOOGLE_CLIENT_ID").unwrap());
    let client_secret = ClientSecret::new(std::env::var("GOOGLE_CLIENT_SECRET").unwrap());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap();
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap();

    let redirect_url = RedirectUrl::new(
        "http://localhost:3001/auth/google/callback".into()
    ).unwrap();

    let client = BasicClient::new(
        client_id,
        Some(client_secret),
        auth_url,
        Some(token_url)
    ).set_redirect_uri(redirect_url);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .url();

    (StatusCode::FOUND, [(axum::http::header::LOCATION, auth_url.to_string())])
}

// ======================================
// GOOGLE CALLBACK HANDLER
// ======================================
pub async fn google_callback_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(params): Query<HashMap<String, String>>
) -> impl IntoResponse {
    // --- Langkah 1: Ambil Authorization Code ---
    let code = match params.get("code") {
        Some(c) => c.clone(),
        None => {
            // ✅ Perbaikan 1: Kembalikan (StatusCode, Json)
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "error", "message": "Kode otorisasi tidak ditemukan"})),
            ).into_response();
        }
    };

    // --- Langkah 2: Inisialisasi OAuth2 Client ---
    let client_id = ClientId::new(std::env::var("GOOGLE_CLIENT_ID").unwrap());
    let client_secret = ClientSecret::new(std::env::var("GOOGLE_CLIENT_SECRET").unwrap());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into()).unwrap();
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".into()).unwrap();
    let redirect_url = RedirectUrl::new(
        "http://localhost:3001/auth/google/callback".into()
    ).unwrap();

    let client = BasicClient::new(
        client_id,
        Some(client_secret),
        auth_url,
        Some(token_url)
    ).set_redirect_uri(redirect_url);

    // --- Langkah 3: Tukar Code dengan Token ---
    let token_result = match
        client.exchange_code(AuthorizationCode::new(code)).request_async(async_http_client).await
    {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error exchange token: {:?}", e);
            // ✅ Perbaikan 2: Kembalikan (StatusCode, Json)
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": "Gagal mengambil token"})),
            ).into_response();
        }
    };

    let access_token = token_result.access_token().secret();

    // --- Langkah 4: Ambil Info User dari Google ---
    let user_info = match
        reqwest::Client
            ::new()
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send().await
    {
        Ok(res) =>
            match res.json::<Value>().await {
                Ok(data) => data,
                Err(_) => {
                    // ✅ Perbaikan 3: Kembalikan (StatusCode, Json)
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"status": "error", "message": "Gagal parsing data user"})),
                    ).into_response();
                }
            }
        Err(_) => {
            // ✅ Perbaikan 4: Kembalikan (StatusCode, Json)
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "error", "message": "Gagal mengambil data user"})),
            ).into_response();
        }
    };

    let email = user_info["email"].as_str().unwrap_or("").to_string();
    let name = user_info["name"].as_str().unwrap_or("Pengguna Google").to_string();

    // --- Langkah 5: Upsert User di Database (Model) ---
    if let Err(e) = User::upsert_google_user(&state.db, &email, &name).await {
        eprintln!("Error saat upsert Google user: {:?}", e);
        // Jika upsert gagal, kembalikan error 500
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": "Gagal menyimpan data pengguna"})),
        ).into_response();
    }

    // --- Langkah 6: Buat dan Encode JWT ---
    let secret = std::env::var("JWT_SECRET").unwrap();
    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::hours(2))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: email.clone(),
        role: "user".to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    ).unwrap();

    // --- Langkah 7: Buat Cookie dan Response Sukses ---
    let cookie = Cookie::build(("jwt", token.clone()))
        .http_only(true)
        .secure(false)
        .path("/")
        .max_age(Duration::hours(2))
        .build();

    let updated_jar = jar.add(cookie);

    let user_data = UserLoginData {
        email: email.clone(),
        role: "user".to_string(),
    };

    // ✅ Perbaikan 5: Return sukses (CookieJar, Json) tanpa .into_response()
    (
        updated_jar,
        Json(LoginResponse {
            status: "success".into(),
            message: "Login berhasil".into(),
            token: Some(token),
            user: Some(user_data),
            
        }),
    ).into_response()
}

// ======================================
// UPDATE ROLE HANDLER
// ======================================
pub async fn update_role_handler(
    State(state): State<Arc<AppState>>,
    Path(email): Path<String>,
    jar: CookieJar,
    Json(payload): Json<UpdateRoleRequest>
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 🔒 Ambil token JWT dari cookie
    let token = match jar.get("jwt") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Err((StatusCode::UNAUTHORIZED, "Token tidak ditemukan".into()));
        }
    };

    // 🔑 Ambil secret dari .env
    let secret = std::env
        ::var("JWT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET tidak diset".into()))?;

    // 🧾 Verifikasi token
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    ).map_err(|_| (StatusCode::UNAUTHORIZED, "Token tidak valid".into()))?;

    // 🕵️‍♀️ Cek role user dari token
    if token_data.claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Kamu bukan admin, gak boleh ubah role!".into()));
    }

    // ✅ Validasi role yang boleh
    if payload.role != "admin" && payload.role != "user" {
        return Err((StatusCode::BAD_REQUEST, "Role tidak valid".into()));
    }

    // 💾 Update role di database menggunakan model
    User::update_role(&state.db, &email, &payload.role).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        e.to_string(),
    ))?;

    Ok((
        StatusCode::OK,
        Json(
            json!({
  "status": "success",
  "message": format!("Role {} berhasil diubah menjadi {}", email, payload.role)
 })
        ),
    ))
}
