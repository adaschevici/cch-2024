use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tera::escape_html;
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Invalid color requested")]
    Teapot,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Teapot => (StatusCode::IM_A_TEAPOT, "Invalid color requested"),
        };

        (status, error_message).into_response()
    }
}
async fn star() -> Result<String, AppError> {
    Ok(r#"<div id="star" class="lit"></div>"#.to_string())
}

async fn flip_present_color(Path(color): Path<String>) -> Result<String, AppError> {
    let new_color = match color.as_str() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => return Err(AppError::Teapot),
    };
    Ok(format!(
        r#"<div class="present {color}" hx-get="/23/present/{new_color}" hx-swap="outerHTML">
           <div class="ribbon"></div>
           <div class="ribbon"></div>
           <div class="ribbon"></div>
           <div class="ribbon"></div>
    </div>"#
    ))
}

async fn flip_ornament_state(Path((state, n)): Path<(String, String)>) -> Result<String, AppError> {
    let n = escape_html(&n);
    let (class, new_state) = match state.as_str() {
        "on" => ("ornament on", "off"),
        "off" => ("ornament", "on"),
        _ => return Err(AppError::Teapot),
    };
    Ok(format!(
        r#"<div class="{class}" id="ornament{n}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{new_state}/{n}" hx-swap="outerHTML"></div>"#,
    ))
}

async fn upload_manifest() {}

pub fn router() -> Router {
    Router::new()
        .route("/star", get(star))
        .route("/present/:color", get(flip_present_color))
        .route("/ornament/:state/:n", get(flip_ornament_state))
        .route("/lockfile", post(upload_manifest))
}
