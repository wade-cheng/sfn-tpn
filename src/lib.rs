//! Saffron's two-player turn-based strategy game networking code.

mod protocol;

use tokio::{
    sync::mpsc::{self, Receiver, Sender, error::TryRecvError},
    task::{self, JoinHandle},
};

/// The interface for netcode.
///
/// Runs Tokio and iroh under the hood in a separate thread. Methods must be
/// called from the context of a Tokio runtime. The procedure for operation
/// is as follows.
///
/// A [`new`][`NetcodeInterface::new`] `NetcodeInterface` should be created on
/// the two players' machines. The first, the "server," must provide
/// `None` ticket. The second, the "client," must provide `Some` ticket string
/// the server reveals. (The server currently prints it to stdout. In the future,
/// this might be relayed through a tokio oneshot thing.)
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
/// Deviations from this procedure are undefined behavior.
pub struct NetcodeInterface {
    is_my_turn: bool,
    recv_from_iroh: Receiver<[u8; 4]>,
    send_to_iroh: Sender<[u8; 4]>,
    /// A handle to the thread running iroh under the hood.
    ///
    /// Might need to be dropped if we want to be pedantic about the code.
    _iroh_handle: JoinHandle<()>,
}

impl NetcodeInterface {
    /// Create a new interface.
    ///
    /// See the struct's [`docs`][`NetcodeInterface`] for invariants.
    pub fn new(ticket: Option<String>) -> Self {
        // hand-coding a bidirectional channel, sorta :p
        let (send_to_iroh, recv_from_game) = mpsc::channel(1);
        let (send_to_game, recv_from_iroh) = mpsc::channel(1);
        let is_my_turn = ticket.is_some();
        let _iroh_handle = task::spawn(protocol::start_iroh_protocol(
            send_to_game,
            recv_from_game,
            ticket,
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
    pub fn send_turn(&mut self, turn: &[u8; 4]) {
        assert!(self.is_my_turn);
        self.send_to_iroh
            .try_send(*turn)
            .expect("we should never have a full buffer");
        self.is_my_turn = false;
    }

    /// Check if the other player has sent a turn to the user.
    ///
    /// See the struct's [`docs`][`NetcodeInterface`] for invariants.
    pub fn try_recv_turn(&mut self) -> Result<[u8; 4], ()> {
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
