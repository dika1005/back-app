use chrono::{Utc, Duration as ChronoDuration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

// Impor Claims dari dtos karena JWT utils menggunakannya
use crate::dtos::auth::Claims;

// Tipe alias untuk memudahkan penanganan error
type Result<T> = std::result::Result<T, (StatusCode, String)>;

/// Mengambil secret dari env dan menghitung waktu kedaluwarsa untuk access token (menit).
fn get_jwt_config_minutes(duration_minutes: i64) -> Result<(EncodingKey, usize)> {
    let secret = std::env
        ::var("JWT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET not set".into()))?;

    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::minutes(duration_minutes))
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Gagal menghitung waktu kedaluwarsa JWT".into()
        ))?
        .timestamp() as usize;

    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    Ok((encoding_key, expiration))
}

/// Mirip get_jwt_config tapi untuk refresh token (hari).
fn get_refresh_config(duration_days: i64) -> Result<(EncodingKey, usize)> {
    let secret = std::env::var("REFRESH_TOKEN_SECRET")
        .unwrap_or_else(|_| std::env::var("JWT_SECRET").unwrap_or_default());

    if secret.is_empty() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "REFRESH_TOKEN_SECRET or JWT_SECRET not set".into()));
    }

    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::days(duration_days))
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Gagal menghitung waktu kedaluwarsa refresh token".into()
        ))?
        .timestamp() as usize;

    Ok((EncodingKey::from_secret(secret.as_bytes()), expiration))
}

/// Struktur claim khusus untuk refresh token (tersedia publik agar handler bisa memeriksa)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: usize,
    pub typ: String, // harus "refresh"
}

/// Fungsi untuk membuat dan meng-encode JWT akses (durasi dalam menit).
pub fn create_jwt(sub: String, role: String, duration_minutes: i64) -> Result<String> {
    let (encoding_key, expiration) = get_jwt_config_minutes(duration_minutes)?;

    let claims = Claims {
        sub,
        role,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Membuat refresh token (string) dengan durasi dalam hari.
pub fn create_refresh_token(sub: String, duration_days: i64) -> Result<String> {
    let (encoding_key, expiration) = get_refresh_config(duration_days)?;

    let claims = RefreshClaims { sub, exp: expiration, typ: "refresh".into() };

    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Verifikasi token JWT akses dan kembalikan claims ter-deserialize.
pub fn verify_jwt(token: &str) -> Result<Claims> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET not set".into()))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ).map_err(|_| (StatusCode::UNAUTHORIZED, "Token tidak valid".into()))?;

    Ok(token_data.claims)
}

/// Verifikasi refresh token, pastikan typ == "refresh" dan kembalikan RefreshClaims.
pub fn verify_refresh_token(token: &str) -> Result<RefreshClaims> {
    let secret = std::env::var("REFRESH_TOKEN_SECRET")
        .unwrap_or_else(|_| std::env::var("JWT_SECRET").unwrap_or_default());

    if secret.is_empty() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "REFRESH_TOKEN_SECRET or JWT_SECRET not set".into()));
    }

    let token_data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ).map_err(|_| (StatusCode::UNAUTHORIZED, "Refresh token tidak valid".into()))?;

    if token_data.claims.typ != "refresh" {
        return Err((StatusCode::UNAUTHORIZED, "Token bukan tipe refresh".into()));
    }

    Ok(token_data.claims)
}