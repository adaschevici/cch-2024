use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result},
    routing::post,
    Json, Router,
};
use leaky_bucket::RateLimiter;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::sync::Mutex;

type MilkBucket = Arc<Mutex<RateLimiter>>;

const BUCKET_INITIAL: usize = 5;
const BUCKET_CAPACITY: usize = 5;
const BUCKET_REFILL_RATE: usize = 1;

#[derive(Error, Debug)]
enum AppError {
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Missing Content-Type header")]
    MissingContentType,

    #[error("Too many requests")]
    TooManyRequests,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::JsonParseError(_) => (StatusCode::NO_CONTENT, "Failed to parse JSON"),
            AppError::MissingContentType => (StatusCode::BAD_REQUEST, "Invalid Content-Type"),
            AppError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "No milk available\n"),
        };

        (status, error_message).into_response()
    }
}

async fn get_milk(
    State(bucket): State<MilkBucket>,
    headers: HeaderMap,
) -> Result<String, AppError> {
    let got_milk = bucket.lock().await.try_acquire(1);
    if !got_milk {
        return Err(AppError::TooManyRequests);
    }
    // let content_type = headers
    //     .get(axum::http::header::CONTENT_TYPE)
    //     .ok_or(AppError::MissingContentType)?;
    Ok("Milk withdrawn\n".to_string())
}

fn get_rate_limiter() -> RateLimiter {
    RateLimiter::builder()
        .initial(BUCKET_INITIAL)
        .max(BUCKET_CAPACITY)
        .interval(Duration::from_secs(BUCKET_REFILL_RATE as u64))
        .build()
}

pub fn router() -> Router {
    let rate_limiter = Arc::new(Mutex::new(get_rate_limiter()));
    Router::new()
        .route("/milk", post(get_milk))
        .with_state(rate_limiter.clone())
}
