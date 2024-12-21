use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{delete, get, post, put},
    Json, Router,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool, Row};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
    token_map: Arc<RwLock<HashMap<String, i64>>>,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Quote Not Found")]
    QuoteNotFound,

    #[error("Invalid ID: {0}")]
    InvalidID(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Pagination error")]
    PaginationError,
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
            AppError::PaginationError => (StatusCode::BAD_REQUEST, "Pagination error".to_string()),
        };

        (status, error_message).into_response()
    }
}
async fn reset_db(State(state): State<Arc<AppState>>) -> Result<StatusCode, AppError> {
    let transaction = state.pool.begin().await.unwrap();
    sqlx::query("TRUNCATE TABLE quotes;")
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
    author: String,
    quote: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
struct QuoteRecord {
    id: Uuid,
    author: String,
    quote: String,
    version: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}
// impl Quote {
//     fn new(author: String, quote: String) -> Self {
//         Self {
//             id: Uuid::new_v4(),
//             author,
//             quote,
//         }
//     }
// }

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
) -> Result<Json<QuoteRecord>, AppError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| AppError::InvalidID("Invalid ID".to_string()))?;

    let quote = sqlx::query_as::<_, QuoteRecord>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::QuoteNotFound,
            _ => AppError::DatabaseError(e),
        })?;

    Ok(Json(quote)) // wtf ???
}

async fn delete_quote_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<QuoteRecord>, AppError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| AppError::InvalidID("Invalid ID".to_string()))?;
    let quote = sqlx::query_as::<_, QuoteRecord>("DELETE FROM quotes WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::QuoteNotFound,
            _ => AppError::DatabaseError(e),
        })?;
    Ok(Json(quote)) // wtf ???
}

async fn update_quote_by_id_increment_version(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(quote): Json<Quote>,
) -> Result<Json<QuoteRecord>, AppError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| AppError::InvalidID("Invalid ID".to_string()))?;
    let quote = sqlx::query_as::<_, QuoteRecord>(
        "UPDATE quotes SET author = $2, quote = $3, version = version + 1
            WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(quote.author)
    .bind(quote.quote)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::QuoteNotFound,
        _ => AppError::DatabaseError(e),
    })?;
    Ok(Json(quote))
}

async fn add_quote_with_random_uuid_id(
    State(state): State<Arc<AppState>>,
    Json(quote): Json<Quote>,
) -> Result<(StatusCode, Json<QuoteRecord>), AppError> {
    let new_id = Uuid::new_v4();
    let quote = sqlx::query_as::<_, QuoteRecord>(
        "INSERT INTO quotes (id, author, quote) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(new_id)
    .bind(quote.author)
    .bind(quote.quote)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::DatabaseError)?;
    Ok((StatusCode::CREATED, Json(quote))) // wtf ???
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
struct Page {
    page: i64,
    quotes: Vec<QuoteRecord>,
    next_token: Option<String>,
}

#[derive(Deserialize)]
struct QueryParams {
    token: Option<String>,
}

fn generate_cursor() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect::<String>()
}

async fn paginated_quotes_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Page>, AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(id) from quotes")
        .fetch_one(&state.pool)
        .await?;
    let page_number: i64 = if let Some(token) = params.token {
        println!("{:?}", token);
        let mut map = state.token_map.write().await;
        let number = map
            .get(token.as_str())
            .map(|i| *i)
            .ok_or(AppError::PaginationError)?;
        map.remove(token.as_str());
        number
    } else {
        0
    };
    let offset = page_number * 3;
    let next_token = if offset + 3 >= count {
        None
    } else {
        let token = generate_cursor();
        state
            .token_map
            .write()
            .await
            .insert(token.clone(), page_number + 1);
        Some(token)
    };

    let quotes = sqlx::query_as(
        "
            SELECT id,author,quote,created_at,version
            FROM quotes
            ORDER BY created_at ASC
            LIMIT 3 OFFSET $1
            ",
    )
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|err| AppError::DatabaseError(err))?;

    Ok(Json(Page {
        quotes,
        page: page_number + 1,
        next_token,
    }))
}
pub fn router(pool: PgPool) -> Router {
    let shared_state = Arc::new(AppState {
        pool,
        token_map: Arc::new(RwLock::new(HashMap::new())),
    });
    Router::new()
        .route("/reset", post(reset_db))
        .route("/cite/:id", get(get_quote_by_id))
        .route("/remove/:id", delete(delete_quote_by_id))
        .route("/undo/:id", put(update_quote_by_id_increment_version))
        .route("/draft", post(add_quote_with_random_uuid_id))
        .route("/list", get(paginated_quotes_list))
        // .route("/list-notmine", get(list))
        .with_state(shared_state)
}
