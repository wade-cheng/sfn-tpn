//! saffron's two-player networking code for turn-based games.
//!
//! The changelog is currently the commit log. A proper changelog may be added in the future.
//!
//! # What sfn-tpn is made for
//!
//! This crate provides a tiny interface for adding multiplayer to casual two-player turn-based games.
//!
//! "Casual" is important because we connect two players directly and trust the bytes they send.
//! "Hacking" in these games is not an issue because you would simply not accept game invitations from
//! people you do not want to play with.
//!
//! "Two-player turn-based" specifically means two-player games that have strict turns.
//! That is, each player is allowed to take a turn if and only if it is not the other player's turn,
//! and players alternate turns.
//!
//! Examples include chess, checkers, Connect 4, (two-player) Blokus, and such.
//! Nonexamples could include games that allow actions on the other player's turn, like activating Trap Cards
//! in Yu-Gi-Oh, though you might be able to define the concept of a turn such that it works with
//! sfn-tpn.
//!
//! # What sfn-tpn can do
//!
//! This crate exposes a [`NetcodeInterface`](https://docs.rs/sfn_tpn/latest/sfn_tpn/struct.NetcodeInterface.html) with functionality for
//!
//! - connecting two game instances (peer-to-peer via [iroh](https://www.iroh.computer/))
//! - sending byte buffers of a constant size between the two game instances
//! - doing so in a strictly turn-based manner (as described above)
//!
//! # What sfn-tpn cannot do
//!
//! I promise to "do my best" regarding security and not leaking resources, but I do not
//! guarantee everything is perfect. Please feel encouraged to read the source code to make sure
//! any risks or inefficiencies are tolerable for your use case (they are for mine, else
//! I'd have fixed the code). Issues and PRs are appreciated, if you'd like!
//!
//! Additionally, these features are currently considered out of scope for sfn-tpn:
//!
//! - connecting multiple game instances
//! - anything not turn-based
//! - wasm is probably not supported because we use threading
//!   - (I'd like it to be, to be able to use this with macroquad for wasm, but this spawns a host of issues :/)
//!
//! # Examples
//!
//! - See the examples directory at <https://github.com/wade-cheng/sfn-tpn>

mod protocol;

use tokio::{
    sync::{
        mpsc::{self, error::TryRecvError},
        oneshot::{self},
    },
    task::{self, JoinHandle},
};

/// Config used to create a new [`NetcodeInterface`].
///
/// The user was either given a ticket, or is generating a new ticket.
pub enum Config {
    /// A ticket string obtained from the other player.
    Ticket(String),
    /// Holds a oneshot sender that will send a newly generated ticket.
    TicketSender(oneshot::Sender<String>),
}

/// The interface for netcode.
///
/// Runs [Tokio](https://tokio.rs/) and [iroh](https://www.iroh.computer/)
/// under the hood in a separate thread. So, methods must be called from the
/// context of a Tokio runtime. The procedure for operation is as follows.
///
/// A [`new`][`NetcodeInterface::new`] `NetcodeInterface` should be created on
/// the two players' machines. The first, the "server," must provide a oneshot
/// sender that receives a newly generated ticket. The second, the "client,"
/// must provide a ticket string from that server.
///
/// The server moves second and the client moves first.
///
/// If it is the user's turn, they may:
///
/// - [`send_turn`][`NetcodeInterface::send_turn`] once
/// - it will no longer be the user's turn
///
/// If it is not the user's turn, they may:
///
/// - [`try_recv_turn`][`NetcodeInterface::try_recv_turn`] repeatedly
/// - if it returns `Ok`, it will be the user's turn.
///
/// Turns are represented as byte buffers of a constant size. Both players'
/// buffer sizes must be the same.
///
/// Deviations from this procedure are undefined behavior.
pub struct NetcodeInterface<const SIZE: usize> {
    is_my_turn: bool,
    recv_from_iroh: mpsc::Receiver<[u8; SIZE]>,
    send_to_iroh: mpsc::Sender<[u8; SIZE]>,
    /// A handle to the thread running iroh under the hood.
    ///
    /// Might need to be dropped if we want to be pedantic about the code.
    _iroh_handle: JoinHandle<()>,
}

impl<const SIZE: usize> NetcodeInterface<SIZE> {
    /// Create a new interface.
    ///
    /// See the struct's [`docs`][`NetcodeInterface`] for invariants.
    pub fn new(config: Config) -> Self {
        // hand-coding a bidirectional channel, sorta :p
        let (send_to_iroh, recv_from_game) = mpsc::channel(1);
        let (send_to_game, recv_from_iroh) = mpsc::channel(1);
        let is_my_turn = match &config {
            Config::Ticket(_) => true,
            Config::TicketSender(_) => false,
        };
        let _iroh_handle = task::spawn(protocol::start_iroh_protocol(
            send_to_game,
            recv_from_game,
            config,
        ));

        Self {
            is_my_turn,
            _iroh_handle,
            recv_from_iroh,
            send_to_iroh,
        }
    }

    /// Send a turn to the other player.
    ///
    /// See the struct's [`docs`][`NetcodeInterface`] for invariants.
    pub fn send_turn(&mut self, turn: &[u8; SIZE]) {
        assert!(self.is_my_turn);
        self.send_to_iroh
            .try_send(*turn)
            .expect("we should never have a full buffer");
        self.is_my_turn = false;
    }

    /// Check if the other player has sent a turn to the user.
    ///
    /// See the struct's [`docs`][`NetcodeInterface`] for invariants.
    pub fn try_recv_turn(&mut self) -> Result<[u8; SIZE], ()> {
        assert!(!self.is_my_turn);
        match self.recv_from_iroh.try_recv() {
            Ok(t) => {
                self.is_my_turn = true;
                Ok(t)
            }
            Err(TryRecvError::Empty) => Err(()),
            Err(TryRecvError::Disconnected) => unreachable!("unreachable if all goes well"),
        }
    }

    /// Return whether it is the user's turn.
    pub fn my_turn(&self) -> bool {
        self.is_my_turn
    }
}
