use std::{ops::Deref, sync::Arc, time::Duration};

use anyhow::Result;
use axum_macros::FromRequest;
use error::AppError;
use sqlx::{postgres::PgPoolOptions, PgPool};

mod domain;
mod error;
mod handler;
pub mod router;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
pub struct AppStateInner {
    pub app_addr: String,
    pub db: PgPool,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(app_addr: &str, url: String) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(10))
            .max_connections(10)
            .connect(&url)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS short_url (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
                created_at TIMESTAMPTZ NOT NULL default now(),
                updated_at TIMESTAMPTZ NOT NULL default now()
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                app_addr: app_addr.to_string(),
                db: pool,
            }),
        })
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(T);
