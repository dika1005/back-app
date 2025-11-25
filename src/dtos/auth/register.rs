use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "John Doe",
    "email": "john@example.com",
    "password": "Password123!",
    "alamat": "Jakarta, Indonesia"
}))]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub alamat: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    #[schema(example = "success")]
    pub status: String,
    #[schema(example = "Registrasi berhasil!")]
    pub message: String,
    pub user: Option<UserData>,
}

#[derive(Serialize, ToSchema)]
pub struct UserData {
    #[schema(example = "John Doe")]
    pub name: String,
    #[schema(example = "john@example.com")]
    pub email: String,
    #[schema(example = "Jakarta, Indonesia")]
    pub alamat: Option<String>,
}
