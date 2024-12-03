use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

async fn calculate_ipv5_sum() -> &'static str {
    "The sum of the first 5 positive integers is 15"
}

pub fn router() -> Router {
    Router::new().route("/2", get(calculate_ipv5_sum))
}
