use std::collections::HashSet;
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
use jsonwebtoken::{
    decode, decode_header, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const PUBLIC_KEY: &[u8] = include_bytes!("../assets/keys/public_key.pem");

#[derive(Error, Debug)]
enum AppError {
    #[error("Missing Content-Type header")]
    MissingContentType,

    #[error("Missing Cookie Header")]
    MissingCookie,

    #[error("Decoding error")]
    HeaderDecodingError(jsonwebtoken::errors::Error),

    #[error("Decoding error")]
    DecodingError,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Key missing")]
    KeyMissing,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::MissingContentType => (StatusCode::BAD_REQUEST, "Invalid Content-Type"),
            AppError::MissingCookie => (StatusCode::BAD_REQUEST, "Missing Cookie Header"),
            AppError::KeyMissing => (StatusCode::INTERNAL_SERVER_ERROR, "Key missing"),
            AppError::HeaderDecodingError(err) => {
                eprintln!("Header Decoding error: {err}");
                (StatusCode::BAD_REQUEST, "Header Decoding error")
            }
            AppError::DecodingError => (StatusCode::BAD_REQUEST, "Decoding error"),
            AppError::InvalidSignature => (StatusCode::UNAUTHORIZED, "Invalid signature"),
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

async fn decode_package(jwt_token: String) -> Result<Json<Value>, AppError> {
    let header = match decode_header(&jwt_token) {
        Ok(h) => h,
        Err(e) => {
            return Err(AppError::HeaderDecodingError(e));
        }
    };
    let Ok(key) = DecodingKey::from_rsa_pem(PUBLIC_KEY) else {
        return Err(AppError::KeyMissing);
    };
    let mut validation = Validation::new(header.alg);
    validation.required_spec_claims = HashSet::new();
    validation.validate_exp = false;

    let claim = match decode::<Value>(&jwt_token, &key, &validation) {
        Ok(t) => t.claims,
        Err(e) if e.kind() == &jsonwebtoken::errors::ErrorKind::InvalidSignature => {
            eprintln!("invalid JWT signature");
            return Err(AppError::InvalidSignature);
        }
        Err(e) => {
            eprintln!("problem with decoding JWT token: {e:?}");
            return Err(AppError::DecodingError);
        }
    };

    Ok(Json(claim))
}

pub fn router() -> Router {
    let secret = Arc::new(RwLock::new("very_secret_key".to_string()));
    Router::new()
        .route("/wrap", post(wrap_package))
        .route("/unwrap", get(unwrap_package))
        .route("/decode", post(decode_package))
        .with_state(secret)
}
