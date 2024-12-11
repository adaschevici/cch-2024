use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use leaky_bucket::RateLimiter;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

type MilkBucket = Arc<Mutex<RateLimiter>>;

const BUCKET_INITIAL: usize = 5;
const BUCKET_CAPACITY: usize = 5;
const BUCKET_REFILL_RATE: usize = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Payload {
    Liters(f32),
    Litres(f32),
    Gallons(f32),
    Pints(f32),
}
impl Payload {
    fn cal(self) -> Self {
        match self {
            Self::Liters(n) => Self::Gallons(0.264172060 * n),
            Self::Gallons(n) => Self::Liters(3.78541 * n),
            Self::Litres(n) => Self::Pints(1.759754 * n),
            Self::Pints(n) => Self::Litres(0.56826125 * n),
        }
    }
}

async fn get_milk(State(bucket): State<MilkBucket>) -> impl IntoResponse {
    let got_milk = bucket.lock().await.try_acquire(1);
    if !got_milk {
        return (StatusCode::TOO_MANY_REQUESTS, "No milk available\n").into_response();
    }
    (StatusCode::OK, "Milk withdrawn\n").into_response()
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
