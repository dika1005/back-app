use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

pub async fn require_admin<B>(req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let cookie_header = req.headers().get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = cookie_header
        .split("; ")
        .find_map(|c| c.strip_prefix("jwt="))
        .unwrap_or("");

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let secret = std::env::var("JWT_SECRET").unwrap();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256)
    );

    match token_data {
        Ok(data) => {
            if data.claims.role != "admin" {
                return Err(StatusCode::FORBIDDEN);
            }
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
