// src/utils/api_response.rs

use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    // success (digunakan untuk PUT/DELETE success message, tidak ada data)
    pub fn success(msg: &str) -> Self {
        Self {
            status: "success".to_string(),
            message: msg.to_string(),
            data: None,
        }
    }

    // success_data (msg: &str, data: T) - Digunakan untuk GET All/Detail
    pub fn success_data(msg: &str, data: T) -> Self {
        Self {
            status: "success".to_string(),
            message: msg.to_string(),
            data: Some(data),
        }
    }
    
    // success_data_with_message (digunakan untuk POST Create, data + pesan)
    // Tipe argumen disesuaikan dengan handler Anda: (msg: String, data: T)
    pub fn success_data_with_message(msg: String, data: T) -> Self {
        Self {
            status: "success".to_string(),
            message: msg,
            data: Some(data),
        }
    }
    
    // error (generic error helper)
    fn error_base(msg: &str) -> ApiResponse<()> {
        ApiResponse {
            status: "error".to_string(),
            message: msg.to_string(),
            data: None,
        }
    }

    // Helper functions untuk error response (menggunakan ApiResponse<()>)
    pub fn not_found(msg: &str) -> ApiResponse<()> {
        Self::error_base(msg)
    }

    pub fn bad_request(msg: &str) -> ApiResponse<()> {
        Self::error_base(msg)
    }

    pub fn internal_error(msg: &str) -> ApiResponse<()> {
        Self::error_base(msg)
    }
}