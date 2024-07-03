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

impl TileColor {
    pub fn highlight_color(self) -> Color {
        match self {
            TileColor::Red => color_u8!(0xff, 0x70, 0x70, 0xff),
            TileColor::Yellow => color_u8!(0xff, 0xee, 0x75, 0xff),
            TileColor::Green => color_u8!(0x8d, 0xff, 0x6b, 0xff),
            TileColor::Blue => color_u8!(0x28, 0xa0, 0xff, 0xff),
            TileColor::Empty | TileColor::Wall => BLANK,
        }
    }
}

/// Player data
#[derive(Debug, Clone)]
pub struct Player {
    /// Player's color
    pub color: TileColor,
    /// Denotes which pieces this player still has available
    pub remaining_pieces: BitSet<PieceID>,
}

impl Player {
    /// Construct a new player with this color, all pieces in hand.
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
            Player::new(TileColor::Blue),
            Player::new(TileColor::Yellow),
            Player::new(TileColor::Red),
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

        // Place invisible colored square in each corner of the board so players
        // can make the first move. This simplifies move validation and makes
        // bounds-checking less annoying. For two players, we place them across
        // from each other, otherwise each player takes their turn in a
        // clockwise order.
        let corners = if players.len() <= 2 {
            [(21, 21), (0, 0), (0, 0), (0, 0)]
        } else {
            [(21, 21), (21, 0), (0, 0), (0, 21)]
        };
        for (p, (row, col)) in players.iter().zip(corners) {
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

    /// Returns adjusted coordinates if `shape` can be placed at them. Returns `None` otherwise.
    pub fn check_bounds_and_recenter(&self, row: isize, col: isize) -> Option<(isize, isize)> {
        // top row, bottom row, left col, right col
        let mut shape_bounds = [0; 4];
    
        for (dr, r) in self.piece_buffer.iter().enumerate() {
            for dc in r.iter_ones() {
                let dr = dr as isize - 2;
                let dc = dc as isize - 2;
                // Only update if we have any 1s in this row. If we don't, do nothing.
                if dr < shape_bounds[0] {
                    shape_bounds[0] = dr;
                } else if dr > shape_bounds[1] {
                    shape_bounds[1] = dr;
                }
    
                if dc < shape_bounds[2] {
                    shape_bounds[2] = dc;
                } else if dc > shape_bounds[3] {
                    shape_bounds[3] = dc;
                }
            }
        }
    
        // dbg!(&shape_bounds);
    
        if row + shape_bounds[0] >= 0
            && row + shape_bounds[1] < (self.board.len() - 2) as isize
            && col + shape_bounds[2] >= 0
            && col + shape_bounds[3] < (self.board.len() - 2) as isize
        {
            Some((row - 2, col - 2))
        } else {
            None
        }
    }

    /// Writes the current player's piece buffer to the board centered at `row` and `col`.
    pub fn place_piece(&mut self, adj_row: isize, adj_col: isize) {
        debug::print_board(&self.board);
        debug_assert!(!self.players.is_empty());
        let player = &mut self.players[self.current_player];
        for (dr, r) in self.piece_buffer.iter().enumerate() {
            for dc in r.iter_ones() {
                // Sometimes I wish Rust allowed signed indices.
                let r_ind = (adj_row + dr as isize) as usize;
                let c_ind = (adj_col + dc as isize) as usize;
                self.board[r_ind + 1][c_ind + 1] = player.color;
            }
        }

        player.remaining_pieces.remove(self.selected_piece.unwrap());

        self.selected_piece = None;
        self.piece_buffer = piece::EMPTY_SHAPE;
        // We were able to place a piece, so clearly this player did not pass.
        self.pass_counter = 0;
    }

    /// Determines if the current move is valid. Accepts a pointer to the full game board
    /// and the player who wishes to make the move. Assumes the piece will be in bounds.
    pub fn valid_move(&self, adj_row: isize, adj_col: isize) -> bool {
        self._valid_move(&self.piece_buffer, adj_row, adj_col)
    }

    // For internal use -- needed only because `can_make_move` needs its own piece buffer.
    fn _valid_move(&self, piece_buffer: &piece::Shape, adj_row: isize, adj_col: isize) -> bool {
        let player = &self.players[self.current_player];
        let mut any_diagonal_matches = false;

        for (r_ind, row) in piece_buffer.iter().enumerate() {
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

    /// Go to the next player.
    pub fn end_turn(&mut self) {
        self.current_player = (self.current_player + 1) % self.players.len();
    }

    /// With the current implementation of things, a naive solution
    /// is the best one.
    ///
    /// This tries to place all remaining pieces on every tile on the board for eight possible orientations.
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
                    (0..20).any(|row| (0..20).any(|col| self._valid_move(&piece_buf, row, col)))
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

    pub fn select_piece(&mut self, piece_id: Option<PieceID>) {
        self.selected_piece = piece_id;
        let shape = match piece_id {
            Some(id) => piece::SHAPES[id],
            None => piece::EMPTY_SHAPE,
        };
        self.piece_buffer = shape;
    }

    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player]
    }

    #[cfg(test)]
    pub fn try_advance_turn(&mut self, row: usize, col: usize) -> bool {
        let (adj_row, adj_col) =
            match self.check_bounds_and_recenter(row as isize, col as isize) {
                Some(coords) => coords,
                None => return false,
            };

        let place_ok = self.valid_move(adj_row + 1, adj_col + 1);
        if place_ok {
            self.place_piece(adj_row, adj_col);
            self.end_turn();
        }
        place_ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_correct_move() {
        let mut game_state = GameState::new(4);
        // L5 shape, bottom right:
        game_state.select_piece(Some(10));
        assert!(game_state.try_advance_turn(18, 18));

        // Dot piece, bottom left:
        game_state.select_piece(Some(0));
        assert!(game_state.try_advance_turn(19, 0));

        // Notch square, top left:
        game_state.select_piece(Some(14));
        assert!(game_state.try_advance_turn(0, 1));
    }

    #[test]
    fn reject_incorrect_move() {
        // Wrong corner
        // Adjacent, same color
        // Middle of nowhere
    }

    #[test]
    fn decide_if_playable() {
        let mut game_state = GameState::new(2);
        // At least the *empty* board should be considered playable.
        assert!(game_state.can_make_move());

        game_state.select_piece(Some(1));
        game_state.try_advance_turn(18, 19);
        // Nowhere near done.
        assert!(game_state.can_make_move());
    }
}
