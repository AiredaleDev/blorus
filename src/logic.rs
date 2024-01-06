use bit_set::BitSet;
use macroquad::prelude::*;

use crate::{debug, piece};

pub type PieceID = usize;

/// Denotes possible tile colors. Also used to denote player colors.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileColor {
    #[default]
    Empty,
    Red,
    Yellow,
    Green,
    Blue,
    Wall,
}

impl std::fmt::Display for TileColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Red => "R",
            Self::Yellow => "Y",
            Self::Green => "G",
            Self::Blue => "B",
            Self::Empty => ".",
            Self::Wall => "#",
        };
        write!(f, "{}", s)
    }
}

impl Into<Color> for TileColor {
    fn into(self) -> Color {
        match self {
            TileColor::Red => RED,
            TileColor::Yellow => YELLOW,
            TileColor::Green => GREEN,
            TileColor::Blue => BLUE,
            TileColor::Empty | TileColor::Wall => BLANK,
        }
    }
}

/// Player data
#[derive(Debug)]
pub struct Player {
    /// Player's color
    pub color: TileColor,
    /// Denotes which pieces this player still has available
    pub remaining_pieces: BitSet<PieceID>,
}

impl Player {
    /// Construct a new player with this color, all pieces in hand,
    /// and an empty piece buffer.
    pub fn new(color: TileColor) -> Self {
        Self {
            color,
            remaining_pieces: BitSet::from_iter(0..=20),
        }
    }
}

/// The current game state.
///
/// Constructed on game start.
#[derive(Debug)]
pub struct GameState {
    /// The current state of the board.
    pub board: [[TileColor; 22]; 22],
    /// Player data.
    pub players: Vec<Player>,
    /// Points to player whose turn it is.
    /// `0 <= current_player <= 3`
    pub current_player: usize,
    /// Denotes the currently selected piece.
    pub selected_piece: Option<PieceID>,
    /// Piece to place (represented as tile grid instead of ID)
    pub piece_buffer: piece::Shape,
    /// Number of turns passed in a row. If equal to `players.len()` then stops the game.
    pub pass_counter: usize,
}

impl GameState {
    /// For internal testing only.
    pub fn new(player_count: usize) -> Self {
        let players: Vec<_> = [
            Player::new(TileColor::Red),
            Player::new(TileColor::Blue),
            Player::new(TileColor::Yellow),
            Player::new(TileColor::Green),
        ]
        .into_iter()
        .take(player_count)
        .collect();

        Self::with_players(players)
    }

    /// Construct a fresh gamestate with a given set of `players`
    pub fn with_players(players: Vec<Player>) -> Self {
        assert!(players.len() <= 4, "Only up to four players are supported!");

        let mut board = [[TileColor::default(); 22]; 22];
        board[0] = [TileColor::Wall; 22];
        board[21] = [TileColor::Wall; 22];

        for i in 0..22 {
            board[i][0] = TileColor::Wall;
            board[i][21] = TileColor::Wall;
        }

        // Place invisible colored square in each corner of the board so players can make the first move.
        // This simplifies move validation and makes bounds-checking less annoying.
        for (p, (row, col)) in players.iter().zip([(21, 21), (0, 0), (0, 21), (21, 0)]) {
            board[row][col] = p.color;
        }

        Self {
            board,
            players,
            current_player: 0,
            selected_piece: None,
            piece_buffer: piece::EMPTY_SHAPE,
            pass_counter: 0,
        }
    }

    /// Attempt to finish the current player's turn by placing their current
    /// piece at `piece_row` and `piece_col`. If the piece cannot be placed,
    /// no state change occurs.
    pub fn try_advance_turn(&mut self, piece_row: usize, piece_col: usize) {
        debug::print_board(&self.board);

        if self.try_place_piece(piece_row, piece_col) {
            self.pass_counter = 0;
            self.current_player = (self.current_player + 1) % self.players.len();
        }
    }

    /// Writes the current player's piece buffer to the board centered at `row` and `col`.
    /// Returns [`true`] if successful, returns [`false`] if piece was OOB.
    pub fn try_place_piece(&mut self, row: usize, col: usize) -> bool {
        debug_assert!(!self.players.is_empty());

        // REFACTOR: Maybe move the offset code into this function.
        // Abstraction is leaking(?) when the piece module needs to "know" about
        // rows and columns of a board
        eprint!("Mapping ({}, {}) -> ", row, col);
        let (row, col) =
            match piece::check_bounds_and_recenter(self.piece_buffer, row as isize, col as isize) {
                Some(offs) => offs,
                None => return false,
            };
        eprintln!("({}, {})", row, col);

        if !self.valid_move(&self.piece_buffer, row + 1, col + 1) {
            return false;
        }

        let player = &mut self.players[self.current_player];
        for (dr, r) in self.piece_buffer.iter().enumerate() {
            for dc in r.iter_ones() {
                // Sometimes I wish Rust allowed signed indices.
                let r_ind = (row + dr as isize) as usize;
                let c_ind = (col + dc as isize) as usize;
                self.board[r_ind + 1][c_ind + 1] = player.color;
            }
        }

        player.remaining_pieces.remove(self.selected_piece.unwrap());

        self.selected_piece = None;
        self.piece_buffer = piece::EMPTY_SHAPE;

        true
    }

    /// Determines if the current move is valid. Accepts a pointer to the full game board
    /// and the player who wishes to make the move. Assumes the piece will be in bounds.
    pub fn valid_move(&self, piece_buf: &piece::Shape, adj_row: isize, adj_col: isize) -> bool {
        let player = &self.players[self.current_player];
        let mut any_diagonal_matches = false;

        for (r_ind, row) in piece_buf.iter().enumerate() {
            for tile in row.iter_ones() {
                let r_coord = adj_row + r_ind as isize;
                let c_coord = adj_col + tile as isize;

                // The board must have space for all tiles that comprise the piece.
                if self.board[r_coord as usize][c_coord as usize] != TileColor::Empty {
                    return false;
                }

                let adjacents = [
                    (r_coord - 1, c_coord),
                    (r_coord, c_coord - 1),
                    (r_coord + 1, c_coord),
                    (r_coord, c_coord + 1),
                ];

                // No tiles adjacent
                if adjacents
                    .into_iter()
                    .any(|(rc, cc)| self.board[rc as usize][cc as usize] == player.color)
                {
                    return false;
                }

                let diagonals = [
                    (r_coord - 1, c_coord - 1),
                    (r_coord + 1, c_coord - 1),
                    (r_coord - 1, c_coord + 1),
                    (r_coord + 1, c_coord + 1),
                ];

                any_diagonal_matches = any_diagonal_matches
                    || diagonals
                        .into_iter()
                        .any(|(rc, cc)| self.board[rc as usize][cc as usize] == player.color);
            }
        }

        any_diagonal_matches
    }

    /// With the current implementation of things, a naive solution
    /// is the best one.
    ///
    /// This tries to place all remaining pieces on every tile on the board.
    /// At all eight possible orientations.
    /// I figured we might be able to make this faster by storing valid coordinates for each player.
    /// That would require I record this set of points in GameState and send rotated copies of those to each player.
    /// I don't know if there's any value to precomputing these points.
    /// This algorithm, on average, *shouldn't* have to iterate through every piece most of the time.
    /// Players will often save their smaller pieces for later, which are more likely to pass any of these checks
    /// and cause the function to return early. In addition, even if they don't, this function will only struggle
    /// to find a match towards the end of the game, where there are fewer pieces to iterate over to begin with.
    /// This otherwise O(rcp) solution *should* almost never reach its worst-case runtime. But it may cause slowdown in
    /// some pathological cases.
    pub fn can_make_move(&self) -> bool {
        let player = &self.players[self.current_player];
        player.remaining_pieces.iter().any(|pc| {
            let mut piece_buf = piece::SHAPES[pc];
            use piece::{FlipDir, RotateDir};
            // Do people find this hard to understand?
            // I don't, but that's because I'm lambda-brained.
            (0..2).any(|_| {
                piece_buf = piece::flip(piece_buf, FlipDir::Vertical);
                (0..4).any(|_| {
                    piece_buf = piece::rotate(piece_buf, RotateDir::Right);
                    (0..20).any(|row| (0..20).any(|col| self.valid_move(&piece_buf, row, col)))
                })
            })
        })
    }

    pub fn is_game_over(&self) -> bool {
        self.players[self.current_player]
            .remaining_pieces
            .is_empty()
            || self.pass_counter == self.players.len()
    }
}
