# Blorus

A recreation of a board game from my childhood. Playable locally on one machine or online!

## Controls

Click on a piece to select it. Press Q and E to rotate the piece, A and D to flip the piece horizontally, or W and S to flip it vertically.
Then, click on the tile where you want to place the piece's center. 

## Building and running

If you don't have it already, install the Rust programming language. I suggest using [Rustup](https://rustup.rs/), which can also be found
in many Linux package repos. Once you've installed the language, clone the repository, enter the folder, and run

```sh
cargo run --release
```

to play the game.

You could also run

```sh
cargo install --release
```

to compile the game and put it into a directory on your path.
