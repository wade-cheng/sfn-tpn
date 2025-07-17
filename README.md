# sfn-tpn

saffron's two-player networking code for turn-based games.

## What sfn-tpn is made for

This crate provides an interface for adding multiplayer to two-player turn-based games.
In particular, it is made for games that have strict turns.
That is, each player is allowed to take a turn if and only if it is not the other player's turn,
and players alternate turns.

Think chess, checkers, Connect 4, (two-player) Blokus, and such.
Nonexamples could include games that allow actions on the other player's turn, like Trap Cards
from Yu-Gi-Oh, though you might be able to define the concept of a turn such that it works with
sfn-tpn.

## What sfn-tpn can do

This crate exposes a [`NetcodeInterface`](https://docs.rs/sfn_tpn/latest/sfn_tpn/struct.NetcodeInterface.html) with functionality for

- connecting two game instances (peer-to-peer via [iroh](https://www.iroh.computer/))
- sending byte buffers of a constant size between the two game instances
- doing so in a strictly turn-based manner (as described above)

## What sfn-tpn can not do

- connect multiple game instances
- anything not turn-based
- run on systems not supported by Tokio and iroh
  - in particular, wasm is not supported because of threading shenanigans

## Examples

- See the examples directory at <https://github.com/wade-cheng/sfn-tpn>
- <https://github.com/wade-cheng/pieceboard>
