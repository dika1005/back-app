use axum::extract::FromRequestParts;
use axum::http::{ request::Parts, StatusCode };
use axum_extra::extract::cookie::CookieJar; // <--- ini penting
use jsonwebtoken::{ decode, DecodingKey, Validation, Algorithm };
use serde::{ Deserialize, Serialize };
use std::{ env, future::Future };

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

#[derive(Clone, Debug)]
pub struct AdminAuth(pub AuthUser); // Wrapper untuk AuthUser

#[allow(refining_impl_trait)]
impl<S> FromRequestParts<S> for AuthUser where S: Send + Sync {
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        // pakai async block biasa
        async move {
            // --- Ambil cookie dari header ---
            let jar = CookieJar::from_headers(&parts.headers);

            // Ambil token dari header Authorization dulu (kalau ada)
            // Be a bit more tolerant: check both `Authorization` and `authorization`,
            // accept values with or without the `Bearer ` prefix, and trim whitespace.
            let header_token = parts.headers
                .get("authorization")
                .or_else(|| parts.headers.get("Authorization"))
                .and_then(|v| v.to_str().ok())
                .map(|v| v.trim())
                .map(|v| {
                    if let Some(s) = v.strip_prefix("Bearer ") {
                        s.to_string()
                    } else {
                        // allow bare token values as well
                        v.to_string()
                    }
                });

            // Kalau gak ada, coba ambil dari cookie "jwt"
            let cookie_token = jar.get("jwt").map(|c| c.value().to_string());

            // Pilih salah satu token yang ketemu
            let token = header_token.or(cookie_token);

            let token = match token {
                Some(t) if !t.is_empty() => t,
                _ => {
                    return Err((StatusCode::UNAUTHORIZED, "Token tidak ditemukan".to_string()));
                }
            };

            // --- Verifikasi token ---
            let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

            let token_data = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::new(Algorithm::HS256)
            ).map_err(|_| (StatusCode::UNAUTHORIZED, "Token tidak valid".to_string()))?;

            Ok(AuthUser {
                email: token_data.claims.sub,
                role: token_data.claims.role,
            })
        }
    }
}

#[allow(refining_impl_trait)]
impl<S> FromRequestParts<S>
    for AdminAuth
    where
        S: Send + Sync,
        // Memastikan AuthUser sudah berhasil diekstrak
        AuthUser: FromRequestParts<S, Rejection = (StatusCode, String)>
{
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        state: &S
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // 1. Pertama: coba ambil JWT dari cookie 'jwt' jika tersedia
            let jar = CookieJar::from_headers(&parts.headers);

            if let Some(cookie) = jar.get("jwt") {
                let token = cookie.value().to_string();

                if token.is_empty() {
                    return Err((StatusCode::UNAUTHORIZED, "Token tidak ditemukan".to_string()));
                }

                // Ambil secret dan verifikasi token
                let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

                let token_data = match decode::<Claims>(
                    &token,
                    &DecodingKey::from_secret(secret.as_bytes()),
                    &Validation::new(Algorithm::HS256),
                ) {
                    Ok(data) => data,
                    Err(_) => return Err((StatusCode::UNAUTHORIZED, "Token tidak valid".to_string())),
                };

                // Pastikan role admin
                if token_data.claims.role != "admin" {
                    return Err((StatusCode::FORBIDDEN, "Akses ditolak: Hanya administrator yang diizinkan.".to_string()));
                }

                let auth_user = AuthUser { email: token_data.claims.sub, role: token_data.claims.role };
                return Ok(AdminAuth(auth_user));
            }

            // 2. Jika cookie tidak ada, fallback ke mekanisme AuthUser (cek header Authorization + cookie)
            let auth_user = AuthUser::from_request_parts(parts, state).await?;

            if auth_user.role != "admin" {
                return Err((StatusCode::FORBIDDEN, "Akses ditolak: Hanya administrator yang diizinkan.".to_string()));
            }

            Ok(AdminAuth(auth_user))
        }
    }
}
