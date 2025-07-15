use std::time::Duration;

use anyhow::{Error, Result};
use sfn_tpn::NetcodeInterface;
use tokio::time::sleep;

/// Return whether our process is a client.
///
/// If not, we must be the server.
///
/// Decides based on command line arguments. If no arguments
/// are supplied, we assume the user wants the process to be
/// a server.
fn is_client() -> Result<bool> {
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
        Err(Error::msg(
            "This process cannot be both the client and the server.",
        ))
    } else {
        Ok(is_client)
    }
}

/// Gets the first ticket string from the command line arguments.
fn ticket() -> Result<String> {
    for arg in std::env::args() {
        if let Some(("--ticket", t)) = arg.split_once("=") {
            return Ok(t.to_string());
        }
    }

    Err(Error::msg(
        "No ticket provided. Clients must provide a ticket to find a server.",
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    if is_client()? {
        // create a send side & send a ping
        NetcodeInterface::new(Some(ticket()?));

        loop {
            sleep(Duration::from_secs(1)).await;
        }
    } else {
        // create the receive side
        NetcodeInterface::new(None);

        loop {
            sleep(Duration::from_secs(1)).await;
        }
    }
}
