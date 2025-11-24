use crate::handlers::chatbot::recommend::chatbot_recommend;
use crate::AppState;
use axum::{routing::post, Router};
use std::sync::Arc;

pub fn chatbot_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/recommend", post(chatbot_recommend))
}