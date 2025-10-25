use chrono::{ Utc, Duration as ChronoDuration };
use jsonwebtoken::{ encode, Header, EncodingKey };
use axum::http::StatusCode;

// Impor Claims dari dtos karena JWT utils menggunakannya
use crate::dtos::auth::Claims; 

// Tipe alias untuk memudahkan penanganan error
type Result<T> = std::result::Result<T, (StatusCode, String)>;

/// Mengambil secret dari env dan menghitung waktu kedaluwarsa.
fn get_jwt_config(duration_hours: i64) -> Result<(EncodingKey, usize)> {
    let secret = std::env
        ::var("JWT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET not set".into()))?;

    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::hours(duration_hours))
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Gagal menghitung waktu kedaluwarsa JWT".into()
        ))?
        .timestamp() as usize;

    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    Ok((encoding_key, expiration))
}

/// Fungsi untuk membuat dan meng-encode JWT saat Login/Google Callback.
pub fn create_jwt(sub: String, role: String, duration_hours: i64) -> Result<String> {
    let (encoding_key, expiration) = get_jwt_config(duration_hours)?;

    let claims = Claims {
        sub,
        role,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &encoding_key
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}