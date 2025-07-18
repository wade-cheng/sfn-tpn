# sfn-tpn

saffron's two-player networking code for turn-based games.

The changelog is currently the commit log. A proper changelog may be added in the future.

## What sfn-tpn is made for

This crate provides a tiny interface for adding multiplayer to casual two-player turn-based games.

"Casual" is important because we connect two players directly and trust the bytes they send.
"Hacking" in these games is not an issue because you would simply not accept game invitations from
people you do not want to play with.

"Two-player turn-based" specifically means two-player games that have strict turns.
That is, each player is allowed to take a turn if and only if it is not the other player's turn,
and players alternate turns.

Examples include chess, checkers, Connect 4, (two-player) Blokus, and such.
Nonexamples could include games that allow actions on the other player's turn, like activating Trap Cards
in Yu-Gi-Oh, though you might be able to define the concept of a turn such that it works with
sfn-tpn.

## What sfn-tpn can do

This crate exposes a [`NetcodeInterface`](https://docs.rs/sfn_tpn/latest/sfn_tpn/struct.NetcodeInterface.html) with functionality for

- connecting two game instances (peer-to-peer via [iroh](https://www.iroh.computer/))
- sending byte buffers of a constant size between the two game instances
- doing so in a strictly turn-based manner (as described above)

## What sfn-tpn cannot do

I promise to "do my best" regarding security and not leaking resources, but I do not
guarantee everything is perfect. Please feel encouraged to read the source code to make sure
any risks or inefficiencies are tolerable for your use case (they are for mine, else
I'd have fixed the code). Issues and PRs are appreciated, if you'd like!

Additionally, these features are currently considered out of scope for sfn-tpn:

- connecting multiple game instances
- anything not turn-based
- wasm is probably not supported because we use threading
  - (I'd like it to be, to be able to use this with macroquad for wasm, but this spawns a host of issues :/)

## Examples

- See the examples directory at <https://github.com/wade-cheng/sfn-tpn>
