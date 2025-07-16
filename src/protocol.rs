//! The underlying iroh protocol implementation.
//! The iroh protcol implementation that the interface uses under the hood.

use iroh::Watcher;
use iroh::{Endpoint, NodeAddr};
use iroh_base::ticket::NodeTicket;
use std::str::FromStr;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::Config;

/// ALPN string for the sfn-tpn protocol.
///
/// The ALPN, or application-layer protocol negotiation, is exchanged in the connection handshake,
/// and the connection is aborted unless both nodes pass the same bytestring.
pub const ALPN: &[u8] = b"saffron/sfn-tpn/0";

/// Starts the pieceboard iroh protocol.
pub async fn start_iroh_protocol(
    send_to_game: Sender<[u8; 4]>,
    mut recv_from_game: Receiver<[u8; 4]>,
    config: Config,
) {
    println!("started iroh protocol in new thread");
    match config {
        Config::Ticket(t) => {
            // we are the client, aka sender, aka player with first move.
            // create a client endpoint and connect to a server based on our ticket.
            let client_endpoint = Endpoint::builder().discovery_n0().bind().await.unwrap();
            let host_addr = NodeAddr::from(
                NodeTicket::from_str(&t).expect("The nodeticket could not be parsed"),
            );

            println!("trying to connect to host...");
            let conn = client_endpoint.connect(host_addr, ALPN).await.unwrap();
            let (mut send, mut recv) = conn.open_bi().await.unwrap();

            println!("client opened bi-stream");

            loop {
                // Send the data the game wants to send
                send.write_all(&recv_from_game.recv().await.unwrap())
                    .await
                    .unwrap();

                let mut buf = [0; 4];
                recv.read_exact(&mut buf).await.unwrap();
                send_to_game
                    .try_send(buf)
                    .expect("we should never have a full buffer");
            }
        }
        Config::TicketSender(sender) => {
            // we are the host, aka receiver, aka player with second move.
            let host_endpoint = Endpoint::builder()
                .discovery_n0()
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await
                .unwrap();

            // send our user the ticket string
            sender
                .send(
                    NodeTicket::new(host_endpoint.node_addr().initialized().await.unwrap())
                        .to_string(),
                )
                .unwrap();

            match host_endpoint.accept().await {
                Some(incoming) => {
                    let connection = incoming.await.unwrap();
                    let node_id = connection.remote_node_id().unwrap();
                    println!("accepted connection from {node_id}");
                    let (mut send, mut recv) = connection.accept_bi().await.unwrap();

                    loop {
                        let mut buf = [0; 4];
                        recv.read_exact(&mut buf).await.unwrap();
                        send_to_game
                            .try_send(buf)
                            .expect("we should never have a full buffer");

                        // Send the data the game wants to send
                        send.write_all(&recv_from_game.recv().await.unwrap())
                            .await
                            .unwrap();
                    }
                }
                None => todo!(),
            }
        }
    }
}
