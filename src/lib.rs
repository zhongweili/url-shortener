mod handlers;

use anyhow::{Context, Result};
use axum::{
    routing::{get, post},
    Router,
};
use handlers::{redirect_handler, shorten_handler};
use std::{ops::Deref, sync::Arc};

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub db_url: String,
    pub pool: PgPool,
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(db_url: String) -> Result<Self> {
        let pool = PgPool::connect(&db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner { db_url, pool }),
        })
    }
}

pub async fn get_router(state: AppState) -> Result<Router> {
    let app = Router::new()
        .route("/", post(shorten_handler))
        .route("/:id", get(redirect_handler))
        .with_state(state);

    Ok(app)
}
