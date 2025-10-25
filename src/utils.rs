use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: Option<T>,
} 

impl<T> ApiResponse<T> {
    pub fn success(msg: &str) -> Self {
        Self {
            status: "success".to_string(),
            message: msg.to_string(),
            data: None,
        }
    }

    pub fn success_data(msg: &str, data: T) -> Self {
        Self {
            status: "success".to_string(),
            message: msg.to_string(),
            data: Some(data),
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            status: "error".to_string(),
            message: msg.to_string(),
            data: None,
        }
    }
}
