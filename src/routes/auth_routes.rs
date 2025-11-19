use crate::AppState;
use crate::handlers::auth::{
    google::{google_auth_handler, google_callback_handler},
    login::login_handler,
    logout::logout_handler,
    refresh::refresh_handler,
    register::register_handler,
    update_role::update_role_handler,
};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

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
