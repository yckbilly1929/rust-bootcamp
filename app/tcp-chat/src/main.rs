use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

use tcp_chat::tcp::run_tcp_server;

#[tokio::main()]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:3000";
    let listener: TcpListener = TcpListener::bind(addr).await?;
    info!("tcp server will start on {}", addr);

    run_tcp_server(listener).await
}
