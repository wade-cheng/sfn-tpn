use ggez::{
    GameResult,
    conf::{WindowMode, WindowSetup},
    event,
};

pub mod constants;
pub mod game;
mod logic;

use constants::BOARD_PX;
use game::GameState;

#[tokio::main]
pub async fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez")
        .window_mode(WindowMode::default().dimensions(BOARD_PX, BOARD_PX))
        .window_setup(WindowSetup::default().title("movable pieces on board"));

    let (mut ctx, event_loop) = cb.build()?;

    let state = GameState::new(&mut ctx).await?;

    event::run(ctx, event_loop, state)
}
