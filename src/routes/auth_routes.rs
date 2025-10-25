use axum::{Router, routing::{post, get}};
use std::sync::Arc;
use crate::handlers::auth_handlers::{
    register_handler,
    login_handler,
    logout_handler,
    google_auth_handler,
    google_callback_handler,
    update_role_handler
};
use crate::AppState;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/google", get(google_auth_handler)) // Redirect ke Google
        .route("/google/callback", get(google_callback_handler)) // Callback dari Google
       .route("/update-role/{email}", post(update_role_handler))
}
