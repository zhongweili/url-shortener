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
use tracing::{debug, warn};

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

#[debug_handler]
pub async fn shorten_handler(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = nanoid!(6);
    let url_record = state
        .add_url_entry(data.url.as_str(), &id)
        .await
        .map_err(|e| {
            warn!("got error {} when adding to db", e);
            StatusCode::UNPROCESSABLE_ENTITY
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
) -> Result<impl IntoResponse, StatusCode> {
    let url = state.get_url(&id).await.map_err(|e| {
        warn!("got error {} when getting url", e);
        StatusCode::NOT_FOUND
    })?;
    debug!("get url: {} from: {}", url, id);
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
