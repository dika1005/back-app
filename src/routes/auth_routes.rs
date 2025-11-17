use axum::{Router, routing::{post, get}};
use std::sync::Arc;
use crate::handlers::auth::{
    register::register_handler,
    login::login_handler,
    logout::logout_handler,
    google::{google_auth_handler, google_callback_handler},
    update_role::update_role_handler,
    refresh::refresh_handler,
};
use crate::AppState;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/google", get(google_auth_handler))
        .route("/google/callback", get(google_callback_handler))
        .route("/update-role/{email}", post(update_role_handler))
        .route("/refresh", post(refresh_handler))
}
