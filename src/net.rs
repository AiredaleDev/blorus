use macroquad::prelude::*;
use std::net::{Ipv4Addr, TcpStream};

use crate::logic::Player;

// Maybe define a trait for local game state and game server so you can use the
// same game loop in local multiplayer and online multiplayer?

// The Go guys were right to include interfaces in their language
// (I presume the Rust guys just included them because they like having more ways to abstract things)

enum OnlinePlayer {
    Active { name: String, data: Player },
    Disconnected { name: String, data: Player },
    Spectator { name: String },
}

// TODO: Maybe move this struct? Or don't, if you can make it sufficiently involve the network.
pub struct GameServer {
    connections: Vec<TcpStream>,
    players: Vec<OnlinePlayer>,
}

impl GameServer {
    pub async fn signal_advance_turn(&self) {}
}

pub async fn create_lobby(_port: u32) -> GameServer {
    // We want a thread/task that listens for new players and accepts them ASAP.
    todo!()
}

pub async fn join_lobby(_addr: Ipv4Addr, _port: u32) {
    // Try to connect to this lobby.
    // Eventually: rejoining mid-game makes you a spectator
    // UNLESS your name is on a list of previously playing players.
    // Then you can jump right back in w/o any trouble.
    // The game will skip your turn while you are away.
}
