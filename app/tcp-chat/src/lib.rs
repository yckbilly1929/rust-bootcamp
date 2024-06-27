use std::{net::SocketAddr, sync::Arc};

use dashmap::DashMap;
use domain::{Message, Peer};
use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::warn;

mod domain;
mod handler;
pub mod tcp;

const MAX_MESSAGES: usize = 128;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<domain::Message>>>,
}

impl State {
    pub async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_receiver) = stream.split();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });

        Peer {
            username,
            stream: stream_receiver,
        }
    }

    pub async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                // skip self-sending
                continue;
            }

            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("failed to send message to {}: {}", peer.key(), e);
                self.peers.remove(peer.key());
            }
        }
    }
}
