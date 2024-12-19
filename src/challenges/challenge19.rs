use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool, Row};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Quote Not Found")]
    QuoteNotFound,

    #[error("Invalid ID: {0}")]
    InvalidID(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::QuoteNotFound => (StatusCode::NOT_FOUND, "Quote Not Found".to_string()),
            AppError::DatabaseError(e) => {
                eprintln!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::InvalidID(quote_id) => {
                (StatusCode::BAD_REQUEST, format!("Invalid ID: {}", quote_id))
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

#[derive(FromRow, Serialize, Deserialize, Debug)]
struct Quote {
    id: String,
    author: String,
    quote: String,
}

// async fn get_quote_by_id(
//     Path(id): Path<String>,
//     State(state): State<Arc<AppState>>,
// ) -> Result<Quote, AppError> {
//     let quote = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
//         .bind(&id)
//         .fetch_one(&state.pool)
//         .await?;
//     Ok(quote)
// }

async fn get_quote_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Quote>, AppError> {
    let id: i32 = id
        .parse()
        .map_err(|_| AppError::InvalidID("Invalid ID".to_string()))?;

    let quote = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::QuoteNotFound,
            _ => AppError::DatabaseError(e),
        })?;

    Ok(Json(quote)) // wtf ???
}

pub fn router(pool: PgPool) -> Router {
    let shared_state = Arc::new(AppState { pool });
    Router::new()
        .route("/reset", post(reset_db))
        .route("/cite/:id", get(get_quote_by_id))
        .with_state(shared_state)
}
