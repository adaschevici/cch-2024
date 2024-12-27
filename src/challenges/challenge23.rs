use axum::{
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Missing Content-Type header")]
    MissingContentType,

    #[error("Missing Cookie Header")]
    MissingCookie,

    #[error("Decoding error")]
    HeaderDecodingError(jsonwebtoken::errors::Error),

    #[error("Decoding error")]
    DecodingError,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Key missing")]
    KeyMissing,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::MissingContentType => (StatusCode::BAD_REQUEST, "Invalid Content-Type"),
            AppError::MissingCookie => (StatusCode::BAD_REQUEST, "Missing Cookie Header"),
            AppError::KeyMissing => (StatusCode::INTERNAL_SERVER_ERROR, "Key missing"),
            AppError::HeaderDecodingError(err) => {
                eprintln!("Header Decoding error: {err}");
                (StatusCode::BAD_REQUEST, "Header Decoding error")
            }
            AppError::DecodingError => (StatusCode::BAD_REQUEST, "Decoding error"),
            AppError::InvalidSignature => (StatusCode::UNAUTHORIZED, "Invalid signature"),
        };

        (status, error_message).into_response()
    }
}
async fn star() -> Result<String, AppError> {
    Ok(r#"<div id="star" class="lit"></div>"#.to_string())
}

pub fn router() -> Router {
    Router::new().route("/star", get(star))
}
