use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
// use axum_extra::extract::WithRejection;
use garde::Validate;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, AppJson, AppState};

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ShortenReq {
    #[garde(length(min = 1, max = 256))]
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

pub async fn shorten(
    State(state): State<AppState>,
    // WithRejection(Json(req), _): WithRejection<Json<ShortenReq>, AppError>,
    AppJson(req): AppJson<ShortenReq>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()
        .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
    let ent = state.shorten(req.url.as_str()).await?;

    let body = Json(ShortenRes {
        url: format!("http://{}/{}", state.app_addr, ent.id),
    });

    Ok((StatusCode::CREATED, body))
}

pub async fn redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let url = state.get_url(&id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, url.parse().unwrap());

    Ok((StatusCode::TEMPORARY_REDIRECT, headers))
}
