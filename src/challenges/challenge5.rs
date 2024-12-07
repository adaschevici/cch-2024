use axum::{
    http::{HeaderMap, StatusCode},
    // TODO: This was a mind bender, need to use Result from
    // axum to be able to return it from handler
    response::{IntoResponse, Response, Result},
    routing::post,
    Router,
};
use cargo_manifest::Manifest;
use serde::{Deserialize, Serialize};
use serde_yaml;
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug)]
struct Package {
    package: PackageInfo,
}

#[derive(Deserialize, Serialize, Debug)]
struct PackageInfo {
    metadata: Option<Metadata>,
    keywords: Option<Vec<String>>,
    #[serde(rename = "rust-version")]
    rust_version: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Metadata {
    orders: Vec<Order>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Order {
    item: String,
    quantity: Option<f32>,
    count: Option<u32>,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Missing Content-Type header")]
    MissingContentType,

    #[error("Unsupported media type")]
    UnsupportedMediaType,

    #[error("No Content Returned")]
    NoContent,

    #[error("Invalid Cargo manifest")]
    InvalidManifest,

    #[error("Magic keyword not provided")]
    MagicKeywordNotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::TomlParseError(_) => (StatusCode::NO_CONTENT, "Failed to parse TOML"),
            AppError::JsonParseError(_) => (StatusCode::NO_CONTENT, "Failed to parse JSON"),
            AppError::YamlParseError(_) => (StatusCode::NO_CONTENT, "Failed to parse YAML"),
            AppError::NoContent => (StatusCode::NO_CONTENT, "No Content Returned"),
            AppError::MagicKeywordNotFound => {
                (StatusCode::BAD_REQUEST, "Magic keyword not provided")
            }
            AppError::UnsupportedMediaType => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported media type")
            }
            AppError::MissingContentType => (StatusCode::BAD_REQUEST, "Invalid Content-Type"),
            AppError::InvalidManifest => (StatusCode::BAD_REQUEST, "Invalid manifest"),
        };

        (status, error_message).into_response()
    }
}
async fn extract_toml(headers: HeaderMap, body: String) -> Result<String, AppError> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .ok_or(AppError::MissingContentType)?;

    // Check if the Content-Type is allowed
    // let allowed_content_type = ["application/toml", "application/yaml", "application/json"];
    let content_type_str = content_type.to_str().unwrap().to_lowercase();
    // if !allowed_content_type.contains(&content_type_str.as_str()) {
    //     return Err(AppError::UnsupportedMediaType);
    // }
    if content_type_str == "application/toml" {
        match Manifest::from_slice(body.as_bytes()) {
            Ok(_) => {}
            Err(_e) => return Err(AppError::InvalidManifest),
        }
    }

    let payload: Package = match content_type_str.as_str() {
        "application/toml" => toml::from_str::<Package>(&body)?,
        "application/yaml" => serde_yaml::from_str::<Package>(&body).unwrap(),
        "application/json" => serde_json::from_str::<Package>(&body).unwrap(),
        _ => return Err(AppError::UnsupportedMediaType),
    };

    if !payload
        .package
        .keywords
        .unwrap_or(vec![])
        .contains(&String::from("Christmas 2024"))
    {
        return Err(AppError::MagicKeywordNotFound);
    }
    if content_type_str != "application/toml" {
        if payload.package.rust_version.is_some()
            && payload
                .package
                .rust_version
                .unwrap()
                .parse::<f32>()
                .is_err()
        {
            return Err(AppError::InvalidManifest);
        };
    }
    let response_parts: Vec<String> = payload
        .package
        .metadata
        .unwrap_or(Metadata { orders: vec![] })
        .orders
        .iter()
        .filter_map(|order| {
            if order.quantity.is_some() {
                Some(order)
            } else {
                None
            }
        })
        .filter_map(|order| {
            if order.quantity.unwrap().fract() == 0.0 {
                Some(order)
            } else {
                None
            }
        })
        .map(|order| format!("{}: {}", order.item, order.quantity.unwrap()))
        .collect();
    if response_parts.is_empty() {
        return Err(AppError::NoContent);
    }

    let response = response_parts.join("\n");
    Ok(response)
}

pub fn router() -> Router {
    Router::new().route("/manifest", post(extract_toml))
}
