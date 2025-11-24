use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub status: String,
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