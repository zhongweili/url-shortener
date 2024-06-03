use std::env;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use url_shortener::{get_router, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "127.0.0.1:6688".to_owned();

    let database_url = env::var("DATABASE_URL")?;
    let state = AppState::try_new(database_url).await?;

    let app = get_router(state).await?;
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
