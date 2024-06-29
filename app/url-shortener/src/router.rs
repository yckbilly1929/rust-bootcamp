use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    error::AppError,
    handler::{redirect, shorten},
    AppState,
};

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    Ok(app)
}
