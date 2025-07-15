//! The underlying iroh protocol implementation.
//! The iroh protcol implementation that the interface uses under the hood.

use anyhow::Result;
use iroh::Watcher;
use iroh::{
    Endpoint, NodeAddr,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler, Router},
};
use iroh_base::ticket::NodeTicket;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

/// Starts the pieceboard iroh protocol.
pub async fn start_iroh_protocol(
    send_to_game: Sender<[u8; 4]>,
    recv_from_game: Receiver<[u8; 4]>,
    ticket: Option<String>,
) {
    println!("started iroh protocol in new thread");
    if let Some(t) = ticket {
        // we are the client, aka sender, aka player with first move.
        // create a client endpoint and connect to a server based on our ticket.
        let client_endpoint = Endpoint::builder().discovery_n0().bind().await.unwrap();
        PieceBoard::new(send_to_game, recv_from_game)
            .connect_to_host(
                &client_endpoint,
                NodeAddr::from(
                    NodeTicket::from_str(&t).expect("The nodeticket could not be parsed"),
                ),
            )
            .await
            .unwrap();
    } else {
        // we are the host, aka receiver, aka player with second move.
        let host_endpoint = Endpoint::builder().discovery_n0().bind().await.unwrap();
        let host_router = Router::builder(host_endpoint)
            .accept(
                PIECEBOARD_ALPN,
                PieceBoard::new(send_to_game, recv_from_game),
            )
            .spawn();
        let addr = host_router
            .endpoint()
            .node_addr()
            .initialized()
            .await
            .unwrap();
        println!("server created.");

        println!(
            "hosting game. another player may join with \n\npieceboard client --ticket={}",
            NodeTicket::new(addr)
        );
        loop {
            sleep(Duration::from_secs(1)).await;
        }
    }
}

/// Each protocol is identified by its ALPN string.
///
/// The ALPN, or application-layer protocol negotiation, is exchanged in the connection handshake,
/// and the connection is aborted unless both nodes pass the same bytestring.
pub const PIECEBOARD_ALPN: &[u8] = b"saffron/pieceboard/0";

/// Ping is a struct that holds both the client ping method, and the endpoint
/// protocol implementation
#[derive(Debug)]
pub struct PieceBoard {
    send_to_game: Sender<[u8; 4]>,
    recv_from_game: Receiver<[u8; 4]>,
}

impl PieceBoard {
    /// create a new Ping
    pub fn new(send_to_game: Sender<[u8; 4]>, recv_from_game: Receiver<[u8; 4]>) -> Self {
        Self {
            send_to_game,
            recv_from_game,
        }
    }

    /// Connect to a host.
    ///
    /// Called by the client, aka player with first move.
    pub async fn connect_to_host(&self, client: &Endpoint, host: NodeAddr) -> Result<()> {
        println!("trying to connect to host...");
        let conn = client.connect(host, PIECEBOARD_ALPN).await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        println!("client opened bi-stream");
        // do wahterver
        // Send some data to be pinged
        send.write_all(b"PING").await?;

        println!("client pinged");

        loop {
            // read the response, which must be PONG as bytes
            let mut buf = [0; 4];
            recv.read_exact(&mut buf).await?;
            assert_eq!(&buf, b"PONG");
            println!("client got pong from server.");
        }
    }
}

impl ProtocolHandler for PieceBoard {
    /// Server code for accepting code.
    ///
    /// The returned future runs on a newly spawned tokio task, so it can run as long as
    /// the connection lasts.
    ///
    /// We have not coded checks for if multiple people have tried connecting
    /// to us. That's bad. TODO.
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        // We can get the remote's node id from the connection.
        let node_id = connection.remote_node_id()?;
        println!("accepted connection from {node_id}");

        // we expect the connecting peer to open a single bi-directional stream.
        let (mut send, mut recv) = connection.accept_bi().await?;
        println!("server accepted bistream");

        let mut buf = [0; 4];
        recv.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"PING");
        println!("server got ping");

        // send back "PONG" bytes
        send.write_all(b"PONG")
            .await
            .map_err(AcceptError::from_err)?;
        println!("server sent pong");
        // todo!("start playing game as first turner")
        loop {
            sleep(Duration::from_secs(1)).await;
        }
    }
}
