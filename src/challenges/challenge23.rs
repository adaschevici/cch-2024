use axum::{
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
pub fn router() -> Router {
    let secret = Arc::new(RwLock::new("very_secret_key".to_string()));
    Router::new()
        .route("/wrap", post(wrap_package))
        .route("/unwrap", get(unwrap_package))
        .route("/decode", post(decode_package))
        .with_state(secret)
}
