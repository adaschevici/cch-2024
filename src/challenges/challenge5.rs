use axum::{
    extract::Json,
    http::{HeaderMap, StatusCode},
    // TODO: This was a mind bender, need to use Result from
    // axum to be able to return it from handler
    response::{IntoResponse, Response, Result},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug)]
struct Package {
    package: PackageInfo,
}

#[derive(Deserialize, Serialize, Debug)]
struct PackageInfo {
    metadata: Metadata,
}

#[derive(Deserialize, Serialize, Debug)]
struct Metadata {
    orders: Vec<Order>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Order {
    item: String,
    quantity: u32,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("Missing Content-Type header")]
    MissingContentType,

    #[error("Invalid Content-Type header")]
    InvalidContentType,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::TomlParseError(_) => (StatusCode::BAD_REQUEST, "Failed to parse TOML"),
            AppError::MissingContentType | AppError::InvalidContentType => {
                (StatusCode::BAD_REQUEST, "Invalid Content-Type")
            }
        };

        (status, error_message).into_response()
    }
}
async fn extract_toml(headers: HeaderMap, body: String) -> Result<Json<Package>, AppError> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .ok_or(AppError::MissingContentType)?;

    // Check if the Content-Type is "application/toml"
    if content_type != "application/toml" {
        return Err(AppError::InvalidContentType);
    }
    let payload = toml::from_str::<Package>(&body)?;
    println!("{:?}", payload);
    Ok(Json(payload))
}

pub fn router() -> Router {
    Router::new().route("/manifest", post(extract_toml))
}