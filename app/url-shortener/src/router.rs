use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    error::AppError,
    handler::{batch_shorten, redirect, shorten},
    AppState,
};

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let app = Router::new()
        .route("/", post(shorten))
        .route("/batch", post(batch_shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    Ok(app)
}
