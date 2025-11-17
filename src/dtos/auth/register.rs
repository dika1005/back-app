use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub alamat: Option<String>,
}

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
