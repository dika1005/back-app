use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum_extra::extract::cookie::CookieJar; // <--- ini penting
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::{env, future::Future};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // email user
    pub role: String,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub email: String,
    pub role: String,
}

#[allow(refining_impl_trait)]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        // pakai async block biasa
        async move {
            // --- Ambil cookie dari header ---
            let jar = CookieJar::from_headers(&parts.headers);

            // Ambil token dari header Authorization dulu (kalau ada)
            let header_token = parts
                .headers
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            // Kalau gak ada, coba ambil dari cookie "jwt"
            let cookie_token = jar.get("jwt").map(|c| c.value().to_string());

            // Pilih salah satu token yang ketemu
            let token = header_token.or(cookie_token);

            let token = match token {
                Some(t) if !t.is_empty() => t,
                _ => return Err((StatusCode::UNAUTHORIZED, "Token tidak ditemukan".to_string())),
            };

            // --- Verifikasi token ---
            let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

            let token_data = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::new(Algorithm::HS256),
            )
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Token tidak valid".to_string()))?;

            Ok(AuthUser {
                email: token_data.claims.sub,
                role: token_data.claims.role,
            })
        }
    }
}
