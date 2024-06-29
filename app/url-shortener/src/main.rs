use std::env;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use url_shortener::{router::get_router, AppState};

#[tokio::main()]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let app_port = env::var("APP_PORT")?;
    let addr = format!("0.0.0.0:{}", app_port);
    let db_url = env::var("DB_URL")?;

    let state = AppState::try_new(&addr, db_url).await?;
    let http_router = get_router(state).await?;

    let listener: TcpListener = TcpListener::bind(&addr).await?;
    info!("http server will start on {}", &addr);

    axum::serve(listener, http_router.into_make_service()).await?;

    Ok(())
}
