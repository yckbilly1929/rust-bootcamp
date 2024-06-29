use chrono::{DateTime, Utc};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{error::AppError, AppState};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ShortUrl {
    #[sqlx(default)]
    pub id: String,
    #[sqlx(default)]
    pub url: String,
    #[sqlx(default)]
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub updated_at: DateTime<Utc>,
}

impl AppState {
    pub async fn shorten(&self, url: &str) -> Result<ShortUrl, AppError> {
        loop {
            let id = nanoid!(6);
            let ent =
                sqlx::query_as(r#"INSERT INTO short_url (id, url) VALUES ($1, $2) RETURNING id"#)
                    .bind(&id)
                    .bind(url)
                    .fetch_one(&self.db)
                    .await;
            match ent {
                Ok(short_url) => return Ok(short_url),
                Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                    if let Some(constraint) = e.constraint() {
                        if constraint == "short_url_pkey" {
                            continue;
                        } else if constraint == "short_url_url_key" {
                            return Err(AppError::UrlDuplicated(url.to_owned()));
                        }
                    }

                    return Err(AppError::SqlxError(sqlx::Error::Database(e)));
                }
                Err(e) => return Err(AppError::SqlxError(e)),
            }
        }
    }

    pub async fn batch_shorten(&self, urls: Vec<String>) -> Result<Vec<String>, AppError> {
        // TODO: batch select to verify, and batch reserve id?
        if urls.is_empty() {
            return Err(AppError::InvalidRequest("invalid param length".to_string()));
        }

        let ids = urls.iter().map(|_| nanoid!(6)).collect();

        let ret = sqlx::query(
            r#"
            INSERT INTO short_url (id, url) SELECT * FROM unnest($1::text[], $2::text[])
        "#,
        )
        .bind(&ids)
        .bind(&urls)
        .execute(&self.db)
        .await;

        match ret {
            Ok(_) => {
                // TODO: validate rows affected?
                Ok(ids)
            }
            Err(e) => Err(AppError::SqlxError(e)),
        }
    }

    pub async fn get_url(&self, id: &str) -> Result<String, AppError> {
        let ent = sqlx::query_as::<_, ShortUrl>("SELECT url FROM short_url WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await;

        match ent {
            Ok(short_url) => Ok(short_url.url),
            Err(sqlx::Error::Database(e)) => Err(AppError::SqlxError(sqlx::Error::Database(e))),
            Err(e) => Err(AppError::UnknownError(e.to_string())),
        }
    }
}
