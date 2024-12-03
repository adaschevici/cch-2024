use axum::{routing::get, Router};

async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

pub fn router() -> Router {
    Router::new().route("/-1", get(hello_bird))
}
