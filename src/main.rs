//! Blokus clone written in Rust.
//!
//! This is a board game from my childhood. It's also a nice excuse to get comfortable with using async/await semantics over the network.

use bit_set::BitSet;
use macroquad::prelude::*;

mod debug;
mod piece;

pub type PieceID = usize;

/// Denotes possible tile colors. Also used to denote player colors.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileColor {
    Red,
    Yellow,
    Green,
    Blue,
    #[default]
    Empty,
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

impl TileColor {
    const ALL: [Self; 5] = [
        Self::Red,
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Empty,
    ];
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
    color: TileColor,
    /// Denotes which pieces this player still has available
    remaining_pieces: BitSet<PieceID>,
    /// Piece to place (represented as tile grid instead of ID)
    piece_buffer: piece::Shape,
}

impl Player {
    /// Construct a new player with this color, all pieces in hand,
    /// and an empty piece buffer.
    pub fn new(color: TileColor) -> Self {
        Self {
            color,
            remaining_pieces: BitSet::from_iter(0..=20),
            piece_buffer: piece::EMPTY_SHAPE,
        }
    }
}

/// The current game state.
///
/// Constructed on game start.
#[derive(Debug)]
pub struct GameState {
    /// The current state of the board.
    board: [[TileColor; 22]; 22],
    /// Player data.
    players: Vec<Player>,
    /// Points to player whose turn it is.
    /// `0 <= current_player <= 3`
    current_player: usize,
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
        // If anyone else looks at this, is this code difficult to understand?
        for (p, (row, col)) in players.iter().zip([(21, 21), (0, 0), (0, 21), (21, 0)]) {
            board[row][col] = p.color;
        }

        Self {
            board,
            players,
            current_player: 0,
        }
    }
}

impl GameState {
    pub fn try_advance_turn(&mut self, piece_row: usize, piece_col: usize) {
        debug::print_board(&self.board);

        if self.try_place_piece(piece_row, piece_col) {
            self.current_player = (self.current_player + 1) % self.players.len();
        }
    }

    /// Returns [`true`] if successful, returns [`false`] if piece was OOB.
    pub fn try_place_piece(&mut self, row: usize, col: usize) -> bool {
        debug_assert!(!self.players.is_empty());
        let player = &self.players[self.current_player];

        // REFACTOR: Maybe move the offset code into this function.
        // Abstraction is leaking(?) when the piece module needs to "know" about
        // rows and columns of a board
        eprint!("Mapping ({}, {}) -> ", row, col);
        let (row, col) =
            match piece::check_bounds_and_recenter(player.piece_buffer, row as isize, col as isize)
            {
                Some(offs) => offs,
                None => return false,
            };
        eprintln!("({}, {})", row, col);

        if !valid_move(&self.board, player, row + 1, col + 1) {
            return false;
        }

        for (dr, r) in player.piece_buffer.iter().enumerate() {
            for dc in r.iter_ones() {
                // Sometimes I wish Rust allowed signed indices.
                let r_ind = (row + dr as isize) as usize;
                let c_ind = (col + dc as isize) as usize;
                self.board[r_ind + 1][c_ind + 1] = player.color;
            }
        }

        true
    }

    /// Update the current player's piece buffer based on the result of `f`.
    /// `f` must accept a [`piece::Shape`] and return a [`piece::Shape`].
    pub fn alter_current_piece_buffer(&mut self, f: impl Fn(piece::Shape) -> piece::Shape) {
        // lambda-brained update semantics xd --
        // TODO: Maybe remove, this is kinda silly.
        // Also, no idea how it will perform when compared to a more explicit solution.
        self.players[self.current_player].piece_buffer =
            f(self.players[self.current_player].piece_buffer);
    }
}

/// Determines if the current move is valid. Accepts a pointer to the full game board
/// and the player who wishes to make the move. Assumes the piece will be in bounds.
pub fn valid_move(
    board: &[[TileColor; 22]; 22],
    player: &Player,
    adj_row: isize,
    adj_col: isize,
) -> bool {
    let adjacents = [(-1, 0), (0, -1), (1, 0), (0, 1)];
    let diagonals = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

    // Assert the following for all tiles in the current piece buffer.

    let no_collisions = player.piece_buffer.iter().enumerate().all(|(r_ind, row)| {
        let r_coord = adj_row + r_ind as isize;
        row.iter_ones().all(|tile| {
            let c_coord = adj_col + tile as isize;
            // This tile is not occupied by another piece.
            board[r_coord as usize][c_coord as usize] == TileColor::Empty
                // No tiles adjacent to this one are of the same color.
                && adjacents.iter().all(|(dr, dc)| {
                    eprintln!("Horizontal: checking ({}, {})", r_coord + dr, c_coord + dc);
                    board[(r_coord + dr) as usize][(c_coord + dc) as usize] != player.color
                })
        })
    });

    let any_diagonal_matches = player.piece_buffer.iter().enumerate().any(|(r_ind, row)| {
        let r_coord = adj_row + r_ind as isize;
        row.iter_ones().any(|tile| {
            let c_coord = adj_col + tile as isize;
            // Any tile that touches the corner of this one is of the same color.
            board[r_coord as usize][c_coord as usize] == TileColor::Empty
                && diagonals.iter().any(|(dr, dc)| {
                    eprintln!("Diagonal: checking ({}, {})", r_coord + dr, c_coord + dc);
                    board[(r_coord + dr) as usize][(c_coord + dc) as usize] == player.color
                })
        })
    });

    dbg!(no_collisions, any_diagonal_matches);

    no_collisions && any_diagonal_matches
}

#[macroquad::main("Blorus")]
async fn main() {
    // Modify these to move or scale the board as a proportion of the screen.
    // The board automatically resizes itself with the window.
    const BOARD_SIZE: f32 = 0.5;
    const BOARD_HORIZ_OFFSET: f32 = 0.25;
    const BOARD_VERT_OFFSET: f32 = 0.25;

    let mut game_state = GameState::new(2);
    // test:
    game_state.players[0].piece_buffer = piece::PIECE_SHAPES[10];
    game_state.players[1].piece_buffer = piece::PIECE_SHAPES[19];

    // =================
    //  -- Main loop --
    // =================

    loop {
        clear_background(BEIGE);

        let tile_size = screen_height() * 0.045 * BOARD_SIZE;
        let (board_left_x, board_top_y) = (
            screen_width() * BOARD_SIZE - screen_height() * BOARD_HORIZ_OFFSET,
            screen_height() * BOARD_VERT_OFFSET,
        );
        let (play_area_left_x, play_area_top_y) = (
            board_left_x + screen_height() * 0.05 * BOARD_SIZE,
            board_top_y + screen_height() * 0.05 * BOARD_SIZE,
        );

        // ===============
        //  -- Drawing --
        // ===============

        {
            // Board
            draw_rectangle(
                board_left_x,
                board_top_y,
                screen_height() * BOARD_SIZE,
                screen_height() * BOARD_SIZE,
                GRAY,
            );

            // Draw the colorful tiles
            for row in 0..20 {
                for col in 0..20 {
                    draw_rectangle(
                        play_area_left_x + col as f32 * tile_size,
                        play_area_top_y + row as f32 * tile_size,
                        tile_size,
                        tile_size,
                        game_state.board[row + 1][col + 1].into(),
                    );
                }
            }

            // Board Border
            draw_rectangle_lines(
                board_left_x,
                board_top_y,
                screen_height() * BOARD_SIZE,
                screen_height() * BOARD_SIZE,
                4.,
                BLACK,
            );

            // Play area border
            draw_rectangle_lines(
                play_area_left_x,
                play_area_top_y,
                screen_height() * 0.9 * BOARD_SIZE,
                screen_height() * 0.9 * BOARD_SIZE,
                4.,
                BLACK,
            );

            // grid time
            // vertical lines:
            for i in 1..20 {
                let line_x = play_area_left_x + i as f32 * tile_size;
                draw_line(
                    line_x,
                    play_area_top_y,
                    line_x,
                    play_area_top_y + 20. * tile_size,
                    2.,
                    BLACK,
                );
            }

            // horizontal lines:
            for i in 1..20 {
                let line_y = play_area_top_y + i as f32 * tile_size;
                draw_line(
                    play_area_left_x,
                    line_y,
                    play_area_left_x + 20. * tile_size,
                    line_y,
                    2.,
                    BLACK,
                );
            }

            // piece preview (temp)
            let player = &game_state.players[game_state.current_player];
            for (r_ind, row) in player.piece_buffer.iter().enumerate() {
                for tile in row.iter_ones() {
                    draw_rectangle(
                        tile as f32 * tile_size + 0.05 * screen_width(),
                        r_ind as f32 * tile_size + 0.35 * screen_height(),
                        tile_size,
                        tile_size,
                        player.color.into(),
                    );
                }
            }
        } // drawing section

        // ======================
        //  -- Input handling --
        // ======================

        {
            // click detection rect
            let board_rect = Rect::new(
                play_area_left_x,
                play_area_top_y,
                20. * tile_size,
                20. * tile_size,
            );

            // Flip pieces
            if [KeyCode::A, KeyCode::D, KeyCode::Left, KeyCode::Right]
                .into_iter()
                .any(is_key_pressed)
            {
                use piece::FlipDir;
                game_state.alter_current_piece_buffer(|p| piece::flip(p, FlipDir::Horizontal));
            }

            if [KeyCode::W, KeyCode::S, KeyCode::Up, KeyCode::Down]
                .into_iter()
                .any(is_key_pressed)
            {
                use piece::FlipDir;
                game_state.alter_current_piece_buffer(|p| piece::flip(p, FlipDir::Vertical));
            }

            // Rotate pieces
            if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::PageUp) {
                use piece::RotateDir;
                game_state.alter_current_piece_buffer(|p| piece::rotate(p, RotateDir::Left));
            }

            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::PageDown) {
                use piece::RotateDir;
                game_state.alter_current_piece_buffer(|p| piece::rotate(p, RotateDir::Right));
            }

            // put a piece on the board
            let (mx, my) = mouse_position();
            if board_rect.contains(Vec2::new(mx, my)) && is_mouse_button_pressed(MouseButton::Left)
            {
                let (col, row) = (
                    ((mx - board_rect.x) / tile_size) as usize,
                    ((my - board_rect.y) / tile_size) as usize,
                );
                dbg!(row, col);
                game_state.try_advance_turn(row, col);
            }
        } // input section
        next_frame().await;
    }
}
