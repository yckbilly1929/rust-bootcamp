use std::sync::Arc;

use anyhow::Result;
use tracing::{info, warn};

use crate::{handler::handle_connection, State};

pub async fn run_tcp_server(listener: tokio::net::TcpListener) -> Result<()> {
    let state = Arc::new(State::default());

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("connection accepted from: {}", raddr);

        let state_cloned = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(state_cloned, raddr, stream).await {
                warn!("failed to handle client on {}: {}", raddr, e);
            }
        });
    }
}
