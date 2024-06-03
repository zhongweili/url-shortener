use crate::AppState;
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{header::LOCATION, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Deserialize)]
pub struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
    clicks: i32,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
    #[sqlx(default)]
    clicks: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyError {
    error: String,
}

#[derive(Error, Debug)]
pub enum UrlError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("url already existed: {0}")]
    UrlExisted(String),
    #[error("url not found: {0}")]
    UrlNotFound(String),
}

impl IntoResponse for UrlError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UrlExisted(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::UrlNotFound(_) => StatusCode::NOT_FOUND,
        };
        (
            status_code,
            Json(MyError {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[debug_handler]
pub async fn shorten_handler(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, UrlError> {
    let id = nanoid!(6);
    let url_record = state
        .add_url_entry(data.url.as_str(), &id)
        .await
        .map_err(|e| {
            warn!("got error {} when adding to db", e);
            UrlError::UrlExisted(data.url)
        })?;

    let body = Json(ShortenRes {
        url: format!("http://127.0.0.1:6688/{}", url_record.id),
        clicks: url_record.clicks,
    });
    Ok((StatusCode::CREATED, body))
}

pub async fn redirect_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, UrlError> {
    let url = state.get_url(&id).await.map_err(|e| {
        warn!("got error {} when getting url", e);
        UrlError::UrlNotFound(id)
    })?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn add_url_entry(&self, url: &str, id: &str) -> Result<UrlRecord> {
        let url_record: UrlRecord =
            sqlx::query_as("INSERT INTO urls (id, url) VALUES ($1, $2) RETURNING id")
                .bind(id)
                .bind(url)
                .fetch_one(&self.pool)
                .await?;
        Ok(url_record)
    }

    async fn get_url(&self, id: &str) -> Result<String> {
        sqlx::query("UPDATE urls SET clicks = clicks + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        let url_record: UrlRecord =
            sqlx::query_as("SELECT id, url, clicks FROM urls WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(url_record.url)
    }
}
