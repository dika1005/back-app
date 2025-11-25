use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "john@example.com",
    "password": "Password123!"
}))]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "success")]
    pub status: String,
    #[schema(example = "Login berhasil!")]
    pub message: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user: Option<UserLoginData>,
}

#[derive(Serialize, ToSchema)]
pub struct UserLoginData {
    #[schema(example = "john@example.com")]
    pub email: String,
    #[schema(example = "user")]
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}
