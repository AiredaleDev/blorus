use macroquad::prelude::*;
use std::net::{Ipv4Addr, TcpStream};

use crate::logic::Player;

// Maybe define a trait for local game state and game server so you can use the
// same game loop in local multiplayer and online multiplayer?

enum OnlinePlayer {
    Playing {
        name: String,
        connection: TcpStream,
        data: Player,
    },
    Spectator {
        name: String,
        connection: TcpStream,
    },
}

impl OnlinePlayer {
    pub fn disconnected(&self) -> bool {
        let mut scratch = [0u8; 1];
        match self {
            Self::Playing { connection, .. } | Self::Spectator { connection, .. } => {
                // we don't care about what packets we see, just that we can't get any more.
                connection.peek(&mut scratch).is_err()
            }
        }
    }
}

// TODO: Maybe move this struct? Or don't, if you can make it sufficiently involve the network.
pub struct GameServer {
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
