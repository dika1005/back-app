use crate::AppState;
use crate::dtos::chatbot::{ChatRequest, ChatResponse, GroqChatRequest, GroqMessage, GroqChatResponse};
use crate::dtos::product::RodProduct;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

pub async fn chatbot_recommend(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    // Ambil semua produk dari DB
    let products = match RodProduct::find_all_details(&state.db).await {
        Ok(prods) => prods,
        Err(e) => {
            eprintln!("Error fetching products: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatResponse {
                    status: "error".to_string(),
                    message: "Gagal mengambil data produk".to_string(),
                    recommendation: String::new(),
                }),
            ).into_response();
        }
    };

    // Format produk untuk system prompt
    let products_context = products
        .iter()
        .map(|p| {
            format!(
                "- {} ({}): {} | Panjang: {} | Line: {} | Cast: {} | Action: {} | Material: {} | Power: {} | Reel: {} | Harga: Rp {}",
                p.name,
                p.category_name,
                p.description,
                p.rod_length.as_deref().unwrap_or("-"),
                p.line_weight.as_deref().unwrap_or("-"),
                p.cast_weight.as_deref().unwrap_or("-"),
                p.action.as_deref().unwrap_or("-"),
                p.material.as_deref().unwrap_or("-"),
                p.power.as_deref().unwrap_or("-"),
                p.reel_size.as_deref().unwrap_or("-"),
                p.price as i64  // Remove comma formatting
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = format!(
        "Kamu adalah asisten penjualan joran pancing yang ramah dan ahli. \
        Berikut adalah katalog produk yang tersedia:\n\n{}\n\n\
        Tugasmu: \n\
        1. Pahami kebutuhan customer (pengalaman, lokasi mancing, budget, target ikan)\n\
        2. Rekomendasikan 1-3 produk yang paling sesuai\n\
        3. Jelaskan kenapa produk tersebut cocok dengan kebutuhan mereka\n\
        4. Gunakan bahasa Indonesia yang natural, ramah, dan friendly\n\
        5. Sebutkan harga dan spesifikasi penting\n\
        6. Jika customer bertanya di luar konteks produk, arahkan kembali ke rekomendasi joran\n\n\
        Jawab dengan singkat (maks 150 kata), jelas, dan fokus pada rekomendasi produk.",
        products_context
    );

    // Panggil Groq API
    let groq_api_key = std::env::var("GROQ_API_KEY")
        .expect("GROQ_API_KEY must be set in .env file");

    let client = reqwest::Client::new();
    let groq_request = GroqChatRequest {
        model: "llama-3.3-70b-versatile".to_string(),
        messages: vec![
            GroqMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            GroqMessage {
                role: "user".to_string(),
                content: payload.message,
            },
        ],
        temperature: 0.7,
        max_tokens: 500,
    };

    let groq_response = match client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", groq_api_key))
        .header("Content-Type", "application/json")
        .json(&groq_request)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error calling Groq API: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatResponse {
                    status: "error".to_string(),
                    message: "Gagal menghubungi AI chatbot".to_string(),
                    recommendation: String::new(),
                }),
            ).into_response();
        }
    };

    let groq_data: GroqChatResponse = match groq_response.json().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing Groq response: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatResponse {
                    status: "error".to_string(),
                    message: "Gagal memproses response AI".to_string(),
                    recommendation: String::new(),
                }),
            ).into_response();
        }
    };

    let recommendation = groq_data
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "Maaf, saya tidak bisa memberikan rekomendasi saat ini.".to_string());

    (
        StatusCode::OK,
        Json(ChatResponse {
            status: "success".to_string(),
            message: "Rekomendasi berhasil".to_string(),
            recommendation,
        }),
    ).into_response()
}