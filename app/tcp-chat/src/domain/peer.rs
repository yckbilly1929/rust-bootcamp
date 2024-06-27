use futures::stream::SplitStream;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Debug)]
pub struct Peer {
    pub username: String,
    pub stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}
