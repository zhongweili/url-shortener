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
use tracing::{info, warn};

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
    long_url: String,
    #[sqlx(default)]
    short_url: String,
    #[sqlx(default)]
    clicks: i32,
}

#[debug_handler]
pub async fn shorten_handler(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, StatusCode> {
    let short_url = nanoid!(6);
    let url_record = state
        .add_url_entry(data.url.as_str(), &short_url)
        .await
        .map_err(|e| {
            warn!("got error {} when adding to db", e);
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    let body = Json(ShortenRes {
        url: format!("http://127.0.0.1:6688/{}", url_record.short_url),
        clicks: url_record.clicks,
    });
    Ok((StatusCode::CREATED, body))
}

pub async fn redirect_handler(
    Path(short_url): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state.get_url(&short_url).await.map_err(|e| {
        warn!("got error {} when getting url", e);
        StatusCode::NOT_FOUND
    })?;
    info!("get url: {} from: {}", url, short_url);
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn add_url_entry(&self, url: &str, short_url: &str) -> Result<UrlRecord> {
        let url_record: UrlRecord = sqlx::query_as(
            "INSERT INTO urls (long_url, short_url) VALUES ($1, $2) RETURNING short_url",
        )
        .bind(url)
        .bind(short_url)
        .fetch_one(&self.pool)
        .await?;
        Ok(url_record)
    }

    async fn get_url(&self, short_url: &str) -> Result<String> {
        sqlx::query("UPDATE urls SET clicks = clicks + 1 WHERE short_url = $1")
            .bind(short_url)
            .execute(&self.pool)
            .await?;

        let url_record: UrlRecord =
            sqlx::query_as("SELECT long_url, short_url, clicks FROM urls WHERE short_url = $1")
                .bind(short_url)
                .fetch_one(&self.pool)
                .await?;

        Ok(url_record.long_url)
    }
}
