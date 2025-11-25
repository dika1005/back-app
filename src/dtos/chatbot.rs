use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "Saya pemula, butuh joran untuk mancing di laut budget 1-2 juta"
}))]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatResponse {
    #[schema(example = "success")]
    pub status: String,
    #[schema(example = "Rekomendasi berhasil")]
    pub message: String,
    pub recommendation: String,
}

// Message untuk Groq - HARUS Deserialize dulu baru Serialize
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroqMessage {
    pub role: String,
    pub content: String,
}

// Request ke Groq API
#[derive(Debug, Serialize)]
pub struct GroqChatRequest {
    pub model: String,
    pub messages: Vec<GroqMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
}

// Response dari Groq API
#[derive(Debug, Deserialize)]
pub struct GroqChatResponse {
    pub choices: Vec<GroqChoice>,
}

#[derive(Debug, Deserialize)]
pub struct GroqChoice {
    pub message: GroqMessage,
}