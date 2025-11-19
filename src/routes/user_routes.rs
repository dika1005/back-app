use axum::{Router, routing::{get, put}};
use crate::handlers::user::profile::{get_profile, update_profile};
use crate::AppState;
use std::sync::Arc;

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/profile", get(get_profile))
        .route("/profile", put(update_profile))
}
