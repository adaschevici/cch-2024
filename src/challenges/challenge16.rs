use tokio::sync::{Mutex, RwLock};

use chrono::{Duration, Utc};
use std::{ops::DerefMut, sync::Arc};
use thiserror::Error;

use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{encode, errors::ErrorKind, DecodingKey, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Error, Debug)]
enum AppError {
    #[error("Missing Content-Type header")]
    MissingContentType,

    #[error("Missing Cookie Header")]
    MissingCookie,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::MissingContentType => (StatusCode::BAD_REQUEST, "Invalid Content-Type"),
            AppError::MissingCookie => (StatusCode::BAD_REQUEST, "Missing Cookie Header"),
        };

        (status, error_message).into_response()
    }
}

type JWTSecret = Arc<RwLock<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claim {
    data: Value,
    exp: usize,
}

async fn wrap_package(
    State(secret): State<JWTSecret>,
    Json(package): Json<Value>,
) -> impl IntoResponse {
    // let content_type = headers
    //     .get(axum::http::header::CONTENT_TYPE)
    //     .ok_or(AppError::MissingContentType)?;
    //
    // // Check if the Content-Type is allowed
    // // let allowed_content_type = ["application/toml", "application/yaml", "application/json"];
    // let content_type_str = content_type.to_str().unwrap().to_lowercase();
    let expiration = Utc::now() + Duration::hours(1);
    let claims = Claim {
        data: package.clone(),
        exp: expiration.timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.read().await.as_bytes()),
    );
    match token {
        Ok(token) => (
            StatusCode::OK,
            [(SET_COOKIE, format!("gift={}; HttpOnly; Path=/", token))],
        ),
        Err(e) => {
            let status = match e.kind() {
                ErrorKind::InvalidRsaKey(_) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            };
            (
                status,
                [(
                    "Error".parse().unwrap(),
                    "Failed to generate token".to_string(),
                )],
            )
        }
    }
    // (
    //     StatusCode::OK,
    //     [(SET_COOKIE, format!("gift={}; HttpOnly; Path=/", token))],
    // )
}

#[axum::debug_handler]
async fn unwrap_package(
    State(secret): State<JWTSecret>,
    jar: CookieJar,
) -> Result<Json<Value>, AppError> {
    let Some(cookie) = jar.get("gift") else {
        eprintln!("missing cookie 'gift'");
        return Err(AppError::MissingCookie);
    };
    let token = cookie.value();
    println!("{}", token);
    let response = match jsonwebtoken::decode::<Claim>(
        token,
        &DecodingKey::from_secret(secret.read().await.as_bytes()),
        &Default::default(),
    ) {
        Ok(claims) => Ok(Json(claims.claims.data)),
        Err(err) => {
            match err.kind() {
                ErrorKind::InvalidToken => Err(AppError::MissingContentType), // Invalid or missing cookie
                _ => Err(AppError::MissingContentType),                       // Unexpected error
            }
        }
    };
    if let Ok(response) = response {
        println!("{:?}", response);
        Ok(response)
    } else {
        response
    }
}

pub fn router() -> Router {
    let secret = Arc::new(RwLock::new("very_secret_key".to_string()));
    Router::new()
        .route("/wrap", post(wrap_package))
        .route("/unwrap", get(unwrap_package))
        .with_state(secret)
}
