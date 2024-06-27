use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, warn};

use crate::{
    domain::{Message, Peer},
    State,
};

pub async fn handle_connection(
    state: Arc<State>,
    raddr: SocketAddr,
    stream: TcpStream,
) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username: ").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer: Peer = state.add(raddr, username, stream).await;

    let message = Arc::new(Message::user_joined(&peer.username));
    info!("joined msg: {}", message);
    state.broadcast(raddr, message).await;

    // on client message
    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("failed to read line from {}: {}", raddr, e);
                break;
            }
        };

        let message = Arc::new(Message::new_chat(&peer.username, &line));
        state.broadcast(raddr, message).await;
    }

    // on exit
    state.peers.remove(&raddr);

    let message = Arc::new(Message::user_left(&peer.username));
    info!("left msg: {}", message);
    state.broadcast(raddr, message).await;

    Ok(())
}
