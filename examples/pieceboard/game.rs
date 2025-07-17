use ggez::{
    Context, GameError, GameResult, event,
    glam::*,
    graphics::{Canvas, Color, DrawMode, Mesh, MeshBuilder, Rect},
    input::mouse::MouseButton,
};
use sfn_tpn::{Config, NetcodeInterface};
use tokio::sync::oneshot;

use crate::{
    constants::TURN_SIZE,
    logic::{Pieces, StateChange, Turn},
};

async fn get_netcode_interface() -> GameResult<NetcodeInterface<TURN_SIZE>> {
    /// Return whether our process is a client.
    ///
    /// If not, we must be the server.
    ///
    /// Decides based on command line arguments. If no arguments
    /// are supplied, we assume the user wants the process to be
    /// a server.
    fn is_client() -> GameResult<bool> {
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
            Err(GameError::CustomError(
                "This process cannot be both the client and the server.".to_string(),
            ))
        } else {
            Ok(is_client)
        }
    }

    /// Gets the first ticket string from the command line arguments.
    fn ticket() -> GameResult<String> {
        for arg in std::env::args() {
            if let Some(("--ticket", t)) = arg.split_once("=") {
                return Ok(t.to_string());
            }
        }

        Err(GameError::CustomError(
            "No ticket provided. Clients must provide a ticket to find a server.".to_string(),
        ))
    }

    if is_client()? {
        Ok(NetcodeInterface::new(Config::Ticket(ticket()?)))
    } else {
        let (send, recv) = oneshot::channel();
        let net = NetcodeInterface::<TURN_SIZE>::new(Config::TicketSender(send));
        println!(
            "hosting game. another player may join with \n\n\
            cargo run --example pieceboard client --ticket={}",
            recv.await.unwrap()
        );
        Ok(net)
    }
}

pub struct GameState {
    board_mesh: Mesh,
    hitcircles_mesh: Mesh,
    drawing_hitcircles: bool,
    pieces: Pieces,
    pieces_mesh: Mesh,
    netcode: NetcodeInterface<TURN_SIZE>,
}

impl GameState {
    /// A mesh that draws the tiles of a board.
    ///
    /// If errors don't happen, the output should be a constant.
    fn board_mesh(ctx: &Context) -> GameResult<Mesh> {
        let mut mb = MeshBuilder::new();

        let mut top = 0;
        let mut left = 1;
        let mut next_row_immediate_dark = true;

        const NUM_TILES: u8 = 8 * 8;
        const NUM_DARK_TILES: u8 = NUM_TILES / 2;

        for _ in 0..NUM_DARK_TILES {
            mb.rectangle(
                DrawMode::fill(),
                Rect::new_i32(100 * left, 100 * top, 100, 100),
                Color::from_rgb(181, 136, 99),
            )?;

            left += 2;
            if left >= 8 {
                left = if next_row_immediate_dark { 0 } else { 1 };
                next_row_immediate_dark = !next_row_immediate_dark;
                top += 1;
            }
        }
        Ok(Mesh::from_data(ctx, mb.build()))
    }

    pub async fn new(ctx: &mut Context) -> GameResult<GameState> {
        let board_mesh = Self::board_mesh(ctx)?;
        let hitcircles_mesh = Pieces::filled().get_mesh(ctx)?;
        let drawing_hitcircles = false;
        let pieces = Pieces::default();
        let pieces_mesh = pieces.get_mesh(ctx)?;
        let netcode = get_netcode_interface().await?;

        Ok(GameState {
            board_mesh,
            hitcircles_mesh,
            drawing_hitcircles,
            pieces,
            pieces_mesh,
            netcode,
        })
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.netcode.my_turn()
            && let Ok(turn) = self.netcode.try_recv_turn()
        {
            self.pieces.do_turn_unchecked(Turn(turn));
            self.pieces_mesh = self.pieces.get_mesh(ctx)?;
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if !self.netcode.my_turn() {
            return Ok(());
        }

        for state_change in self.pieces.handle_click(x, y).unwrap_or(vec![]) {
            match state_change {
                StateChange::Deselected => self.drawing_hitcircles = false,
                StateChange::Selected => self.drawing_hitcircles = true,
                StateChange::PieceMoved(turn) => {
                    self.pieces_mesh = self.pieces.get_mesh(ctx)?;
                    self.netcode.send_turn(&turn.0);
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(240, 217, 181));

        canvas.draw(&self.board_mesh, Vec2::ZERO);
        canvas.draw(&self.pieces_mesh, Vec2::ZERO);
        if self.drawing_hitcircles {
            canvas.draw(&self.hitcircles_mesh, Vec2::ZERO);
        }

        canvas.finish(ctx)?;

        Ok(())
    }
}
