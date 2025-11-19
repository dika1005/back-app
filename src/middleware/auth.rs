use crate::utils::jwt::verify_jwt;
use axum::extract::FromRequestParts;
use axum::http::{StatusCode, request::Parts};
use axum_extra::extract::cookie::CookieJar; // <--- ini penting
use std::future::Future;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub email: String,
    pub role: String,
}

// --- Convenience middleware functions for route layers ---
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode as HttpStatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Middleware that ensures a request has a valid JWT (any role).
/// Returns the inner response when token is valid, otherwise returns 401.
pub async fn auth_user_middleware(req: Request<Body>, next: Next) -> Response {
    // reuse cookie/header parsing logic similar to the extractors above
    let headers = req.headers();
    let jar = CookieJar::from_headers(headers);

    let cookie_token = jar
        .get("jwt")
        .or_else(|| jar.get("token"))
        .map(|c| c.value().to_string());

    let header_token = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim())
        .map(|v| {
            if let Some(s) = v.strip_prefix("Bearer ") {
                s.to_string()
            } else {
                v.to_string()
            }
        });

    let token = cookie_token.or(header_token);

    match token {
        Some(t) if !t.is_empty() => match verify_jwt(&t) {
            Ok(_) => next.run(req).await,
            Err(_) => (
                HttpStatusCode::UNAUTHORIZED,
                "Token tidak valid".to_string(),
            )
                .into_response(),
        },
        _ => (
            HttpStatusCode::UNAUTHORIZED,
            "Token tidak ditemukan".to_string(),
        )
            .into_response(),
    }
}

/// Middleware that ensures the request contains a JWT with role == "admin".
pub async fn admin_auth_middleware(req: Request<Body>, next: Next) -> Response {
    let headers = req.headers();
    let jar = CookieJar::from_headers(headers);

    let cookie_token = jar
        .get("jwt")
        .or_else(|| jar.get("token"))
        .map(|c| c.value().to_string());

    let header_token = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim())
        .map(|v| {
            if let Some(s) = v.strip_prefix("Bearer ") {
                s.to_string()
            } else {
                v.to_string()
            }
        });

    let token = cookie_token.or(header_token);

    match token {
        Some(t) if !t.is_empty() => match verify_jwt(&t) {
            Ok(claims) => {
                if claims.role != "admin" {
                    return (
                        HttpStatusCode::FORBIDDEN,
                        "Akses ditolak: Hanya administrator yang diizinkan.".to_string(),
                    )
                        .into_response();
                }
                next.run(req).await
            }
            Err(_) => (
                HttpStatusCode::UNAUTHORIZED,
                "Token tidak valid".to_string(),
            )
                .into_response(),
        },
        _ => (
            HttpStatusCode::UNAUTHORIZED,
            "Token tidak ditemukan".to_string(),
        )
            .into_response(),
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AdminAuth(pub AuthUser); // Wrapper untuk AuthUser

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
            // Prioritaskan cookie agar client yang sudah login (cookie httpOnly)
            // tidak perlu menambahkan header Authorization pada request POST/PUT/DELETE.
            // Debug: print Cookie header to help identify if client sent it
            if let Some(h) = parts.headers.get("cookie") {
                match h.to_str() {
                    Ok(v) => eprintln!("[auth] Incoming Cookie header: {}", v),
                    Err(_) => eprintln!("[auth] Incoming Cookie header: <non-utf8>"),
                }
            } else {
                eprintln!("[auth] No Cookie header present on request");
            }

            let jar = CookieJar::from_headers(&parts.headers);

            // Coba ambil token dari cookie dulu (nama cookie: `jwt` atau `token`)
            let cookie_token = jar
                .get("jwt")
                .or_else(|| jar.get("token"))
                .map(|c| c.value().to_string());

            // Ambil token dari header Authorization jika cookie tidak tersedia
            // Toleransi: cek `Authorization`/`authorization`, terima nilai dengan atau tanpa
            // prefix `Bearer ` dan trim whitespace.
            let header_token = parts
                .headers
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

            // Pilih token yang ketemu: prefer cookie, lalu header
            let token = cookie_token.or(header_token);

            let token = match token {
                Some(t) if !t.is_empty() => t,
                _ => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        "Token tidak ditemukan".to_string(),
                    ));
                }
            };

            // --- Verifikasi token ---
            let claims = verify_jwt(&token)?;

            Ok(AuthUser {
                email: claims.sub,
                role: claims.role,
            })
        }
    }
}

#[allow(refining_impl_trait)]
impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
    // Memastikan AuthUser sudah berhasil diekstrak
    AuthUser: FromRequestParts<S, Rejection = (StatusCode, String)>,
{
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // 1. Pertama: coba ambil JWT dari cookie 'jwt' jika tersedia
            // Debug: log Cookie header for AdminAuth path as well
            if let Some(h) = parts.headers.get("cookie") {
                match h.to_str() {
                    Ok(v) => eprintln!("[admin_auth] Incoming Cookie header: {}", v),
                    Err(_) => eprintln!("[admin_auth] Incoming Cookie header: <non-utf8>"),
                }
            } else {
                eprintln!("[admin_auth] No Cookie header present on request");
            }

            let jar = CookieJar::from_headers(&parts.headers);

            if let Some(cookie) = jar.get("jwt").or_else(|| jar.get("token")) {
                let token = cookie.value().to_string();

                if token.is_empty() {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        "Token tidak ditemukan".to_string(),
                    ));
                }

                // Verifikasi token menggunakan util pusat
                let claims = match verify_jwt(&token) {
                    Ok(c) => c,
                    Err(e) => return Err(e),
                };

                // Pastikan role admin
                if claims.role != "admin" {
                    return Err((
                        StatusCode::FORBIDDEN,
                        "Akses ditolak: Hanya administrator yang diizinkan.".to_string(),
                    ));
                }

                let auth_user = AuthUser {
                    email: claims.sub,
                    role: claims.role,
                };
                return Ok(AdminAuth(auth_user));
            }

            // 2. Jika cookie tidak ada, fallback ke mekanisme AuthUser (cek header Authorization + cookie)
            let auth_user = AuthUser::from_request_parts(parts, state).await?;

            if auth_user.role != "admin" {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Akses ditolak: Hanya administrator yang diizinkan.".to_string(),
                ));
            }

            Ok(AdminAuth(auth_user))
        }
    }
}
