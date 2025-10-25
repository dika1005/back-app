use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>, // CATATAN: Ini harus di-hash sebelum dikirim ke model!
}