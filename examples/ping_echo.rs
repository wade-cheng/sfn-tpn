//! Send a ping/pong, then echo an incrementing counter.

use std::time::Duration;
use tokio::{sync::oneshot, time::sleep};

use sfn_tpn::{Config, NetcodeInterface};

/// Return whether our process is a client.
///
/// If not, we must be the server.
///
/// Decides based on command line arguments. If no arguments
/// are supplied, we assume the user wants the process to be
/// a server.
fn is_client() -> Result<bool, String> {
    let mut is_client = false;
    let mut is_server = false;
    for arg in std::env::args() {
        if arg == "client" {
            is_client = true;
        }
        if arg == "server" {
            is_server = true;
        }
    }
    if is_client && is_server {
        Err("This process cannot be both the client and the server.".to_string())
    } else {
        Ok(is_client)
    }
}

/// Gets the first ticket string from the command line arguments.
fn ticket() -> Result<String, String> {
    for arg in std::env::args() {
        if let Some(("--ticket", t)) = arg.split_once("=") {
            return Ok(t.to_string());
        }
    }

    Err("No ticket provided. Clients must provide a ticket to find a server.".to_string())
}

/// Naively poll `f` with an argument `a: &mut A` until it returns `Ok`.
async fn wait_for<A, T>(f: fn(&mut A) -> Result<T, ()>, a: &mut A) -> T {
    loop {
        if let Ok(t) = f(a) {
            return t;
        }
        sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    if is_client()? {
        // create a send side & send a ping
        let mut netcode = NetcodeInterface::new(Config::Ticket(ticket()?));
        netcode.send_turn(b"ping");
        println!("Client sent ping");

        assert_eq!(
            b"pong",
            &wait_for(NetcodeInterface::try_recv_turn, &mut netcode).await
        );
        println!("Client recieved pong");

        let mut counter = 0;

        loop {
            let bytes = [0, 0, 0, counter];
            netcode.send_turn(&bytes);
            println!("Client sent {bytes:?}");

            assert_eq!(
                &bytes,
                &wait_for(NetcodeInterface::try_recv_turn, &mut netcode).await
            );
            println!("Client got {bytes:?} back");

            counter += 1;
        }
    } else {
        // create the receive side
        let (send, recv) = oneshot::channel();
        let mut netcode = NetcodeInterface::new(Config::TicketSender(send));

        println!(
            "hosting game. another player may join with \n\n\
            cargo run --example ping_echo client --ticket={}",
            recv.await.unwrap()
        );

        assert_eq!(
            b"ping",
            &wait_for(NetcodeInterface::try_recv_turn, &mut netcode).await
        );
        println!("Server received ping");

        netcode.send_turn(b"pong");
        println!("Server sent pong");

        loop {
            let bytes = wait_for(NetcodeInterface::try_recv_turn, &mut netcode).await;
            println!("Server received: {:?}", &bytes);

            netcode.send_turn(&bytes);
            println!("Server echoed");

            sleep(Duration::from_secs(1)).await;
        }
    }
}
