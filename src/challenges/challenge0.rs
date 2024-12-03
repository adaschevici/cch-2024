use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

async fn redirect_to() -> Response {
    (
        StatusCode::FOUND,
        [(
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        )],
        (), // No response body
    )
        .into_response()
}

pub fn router() -> Router {
    Router::new()
        .route("/-1", get(hello_bird))
        .route("/-1/seek", get(redirect_to))
}
