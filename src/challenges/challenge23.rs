use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response, Result},
    routing::{get, post},
    Router,
};
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use tera::escape_html;
use thiserror::Error;
use toml::{map::Map, Value};

#[derive(Error, Debug)]
enum AppError {
    #[error("Invalid color requested")]
    Teapot,

    #[error("No checksums found")]
    NoChecksums,

    #[error("Invalid checksum")]
    Unprocessable,

    #[error("Missing manifest")]
    MissingManifest,

    #[error("Unusable manifest")]
    UnusableManifest,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Teapot => (StatusCode::IM_A_TEAPOT, "Invalid color requested"),
            AppError::NoChecksums => (StatusCode::BAD_REQUEST, "No checksums found"),
            AppError::Unprocessable => (StatusCode::UNPROCESSABLE_ENTITY, "Invalid checksum"),
            AppError::MissingManifest => (StatusCode::BAD_REQUEST, "Missing manifest"),
            AppError::UnusableManifest => (StatusCode::BAD_REQUEST, "Unusable manifest"),
        };

        (status, error_message).into_response()
    }
}
async fn star() -> Result<Html<String>, AppError> {
    Ok(Html(r#"<div id="star" class="lit"></div>"#.to_string()))
}

async fn flip_present_color(Path(color): Path<String>) -> Result<Html<String>, AppError> {
    let new_color = match color.as_str() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => return Err(AppError::Teapot),
    };
    Ok(Html(format!(
        r#"<div class="present {color}" hx-get="/23/present/{new_color}" hx-swap="outerHTML">
           <div class="ribbon"></div>
           <div class="ribbon"></div>
           <div class="ribbon"></div>
           <div class="ribbon"></div>
    </div>"#
    )))
}

async fn flip_ornament_state(
    Path((state, n)): Path<(String, String)>,
) -> Result<Html<String>, AppError> {
    let n = escape_html(&n);
    let (class, new_state) = match state.as_str() {
        "on" => ("ornament on", "off"),
        "off" => ("ornament", "on"),
        _ => return Err(AppError::Teapot),
    };
    Ok(Html(format!(
        r#"<div class="{class}" id="ornament{n}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{new_state}/{n}" hx-swap="outerHTML"></div>"#,
    )))
}
#[derive(Deserialize)]
struct Package {
    _name: Option<String>,
    _source: Option<String>,
    _version: Option<String>,
    checksum: String,
}

impl Package {
    fn extract_style_tuple(self) -> Option<(String, i32, i32)> {
        let checksum = self.checksum;
        let color = checksum.get(0..6).ok_or("Checksum is too short").ok()?;
        color
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            .then(|| ())?;
        let top = checksum.get(6..8)?;
        let left = checksum.get(8..10)?;
        let top = i32::from_str_radix(top, 16).ok()?;
        let left = i32::from_str_radix(left, 16).ok()?;
        Some((format!("#{color}"), top, left))
    }
}

async fn render_checksums(mut multipart: Multipart) -> Result<Html<String>, AppError> {
    let mut divs = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::MissingManifest)?
    {
        let data = field.text().await.map_err(|_| AppError::MissingManifest)?;
        let payload: Map<String, Value> =
            toml::from_str(&data).map_err(|_| AppError::MissingManifest)?;
        let packages = payload["package"].as_array().unwrap();

        for package in packages {
            if let Ok(payload) = package.clone().try_into::<Package>() {
                let d = payload
                    .extract_style_tuple()
                    .ok_or(AppError::Unprocessable)?;
                divs.push(d);
            }
        }
    }
    if divs.is_empty() {
        return Err(AppError::NoChecksums);
    }
    let html = divs
        .into_iter()
        .map(|(color, top, left)| {
            format!(r#"<div style="background-color:{color};top:{top}px;left:{left}px;"></div>"#)
        })
        .collect();
    Ok(Html(html))
}

pub fn router() -> Router {
    Router::new()
        .route("/star", get(star))
        .route("/present/:color", get(flip_present_color))
        .route("/ornament/:state/:n", get(flip_ornament_state))
        .route("/lockfile", post(render_checksums))
}
