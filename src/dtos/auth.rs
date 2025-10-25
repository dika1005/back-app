use serde::{Deserialize, Serialize};

// --- REQUEST DTOs ---

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub alamat: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

// --- RESPONSE DTOs & JWT CLAIMS ---

#[derive(Serialize)]
pub struct RegisterResponse {
    pub status: String,
    pub message: String,
    pub user: Option<UserData>,
}

#[derive(Serialize)]
pub struct UserData {
    pub name: String,
    pub email: String,
    pub alamat: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub status: String,
    pub message: String,
    pub token: Option<String>,
    pub user: Option<UserLoginData>,
}

#[derive(Serialize)]
pub struct UserLoginData {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}
