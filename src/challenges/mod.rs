use axum::{http::StatusCode, response::IntoResponse, Router};

mod challenge2;
mod challenge5;
mod challenge9;
mod challengeminus1;

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
pub(crate) fn router() -> Router {
    Router::new()
        .nest("/", challengeminus1::router())
        .nest("/2", challenge2::router())
        .nest("/5", challenge5::router())
        .nest("/9", challenge9::router())
}
