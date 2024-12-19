use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Quote Not Found")]
    QuoteNotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::QuoteNotFound => (StatusCode::NOT_FOUND, "Quote Not Found"),
            AppError::DatabaseError(e) => {
                eprintln!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
        };

        (status, error_message).into_response()
    }
}
async fn reset_db(State(state): State<Arc<AppState>>) -> Result<StatusCode, AppError> {
    let transaction = state.pool.begin().await.unwrap();
    sqlx::query("DELETE FROM quotes;")
        .execute(&state.pool)
        .await?;
    transaction.commit().await?;

    // sqlx::query(
    //     "CREATE TABLE IF NOT EXISTS quotes (
    //     id UUID PRIMARY KEY,
    //     author TEXT NOT NULL,
    //     quote TEXT NOT NULL,
    //     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    //     version INT NOT NULL DEFAULT 1
    // );",
    // )
    // .execute(&state.pool)
    // .await
    // .unwrap();

    Ok(StatusCode::OK)
}

pub fn router(pool: PgPool) -> Router {
    let shared_state = Arc::new(AppState { pool });
    Router::new()
        .route("/reset", post(reset_db))
        .with_state(shared_state)
}
