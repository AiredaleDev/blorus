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
    board: [[TileColor; 22]; 22],
    /// Player data.
    players: Vec<Player>,
    /// Points to player whose turn it is.
    /// `0 <= current_player <= 3`
    current_player: usize,
    /// Denotes the currently selected piece.
    selected_piece: Option<PieceID>,
    /// Piece to place (represented as tile grid instead of ID)
    piece_buffer: piece::Shape,
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
        }
    }

    /// Attempt to finish the current player's turn by placing their current
    /// piece at `piece_row` and `piece_col`. If the piece cannot be placed,
    /// no state change occurs.
    pub fn try_advance_turn(&mut self, piece_row: usize, piece_col: usize) {
        debug::print_board(&self.board);

        if self.try_place_piece(piece_row, piece_col) {
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

        if !self.valid_move(row + 1, col + 1) {
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
    pub fn valid_move(&self, adj_row: isize, adj_col: isize) -> bool {
        let player = &self.players[self.current_player];
        let mut any_diagonal_matches = false;

        for (r_ind, row) in self.piece_buffer.iter().enumerate() {
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
    // game_state.piece_buffer = piece::PIECE_SHAPES[10];
    game_state.piece_buffer = piece::SHAPES[19];

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

        // wanted to halve the area so I multiply the side length by sqrt(2)/2.
        let ui_tile_size = tile_size * 0.5 * 1.414;
        // each piece graphic is 5 UI tiles wide, and there are at most 11 per row.
        let avail_pieces_x = 0.5 * screen_width() - 5. * 5.5 * ui_tile_size;
        let avail_pieces_y = 0.8 * screen_height();

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

            let player = &game_state.players[game_state.current_player];
            if game_state.selected_piece.is_some() {
                // piece preview border
                draw_rectangle(
                    0.05 * screen_width() - tile_size,
                    0.35 * screen_height() - tile_size,
                    7. * tile_size,
                    7. * tile_size,
                    GRAY,
                );

                draw_rectangle_lines(
                    0.05 * screen_width() - tile_size,
                    0.35 * screen_height() - tile_size,
                    7. * tile_size,
                    7. * tile_size,
                    4.,
                    BLACK,
                );

                // piece preview
                for (r_ind, row) in game_state.piece_buffer.iter().enumerate() {
                    for tile in row.iter_ones() {
                        draw_rectangle(
                            tile as f32 * tile_size + 0.05 * screen_width(),
                            r_ind as f32 * tile_size + 0.35 * screen_height(),
                            tile_size,
                            tile_size,
                            player.color.into(),
                        );

                        draw_rectangle_lines(
                            tile as f32 * tile_size + 0.05 * screen_width(),
                            r_ind as f32 * tile_size + 0.35 * screen_height(),
                            tile_size,
                            tile_size,
                            2.,
                            BLACK,
                        );
                    }
                }
            }

            // making the "executive" decision not to use the ui library (at least not for this)
            for piece_id in player.remaining_pieces.iter() {
                for (r_ind, row) in piece::SHAPES[piece_id].iter().enumerate() {
                    for tile in row.iter_ones() {
                        let offset = 5. * ui_tile_size;
                        let row = piece_id / 11;
                        let col = piece_id % 11;
                        draw_rectangle(
                            tile as f32 * ui_tile_size + avail_pieces_x + offset * col as f32,
                            r_ind as f32 * ui_tile_size + avail_pieces_y + offset * row as f32,
                            ui_tile_size,
                            ui_tile_size,
                            player.color.into(),
                        );

                        draw_rectangle_lines(
                            tile as f32 * ui_tile_size + avail_pieces_x + offset * col as f32,
                            r_ind as f32 * ui_tile_size + avail_pieces_y + offset * row as f32,
                            ui_tile_size,
                            ui_tile_size,
                            2.,
                            BLACK,
                        );
                    }
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

            let piece_rect = Rect::new(
                avail_pieces_x,
                avail_pieces_y,
                11. * 5. * ui_tile_size,
                10. * ui_tile_size,
            );

            // Flip pieces
            if [KeyCode::A, KeyCode::D, KeyCode::Left, KeyCode::Right]
                .into_iter()
                .any(is_key_pressed)
            {
                use piece::FlipDir;
                game_state.piece_buffer = piece::flip(game_state.piece_buffer, FlipDir::Horizontal);
            }

            if [KeyCode::W, KeyCode::S, KeyCode::Up, KeyCode::Down]
                .into_iter()
                .any(is_key_pressed)
            {
                use piece::FlipDir;
                game_state.piece_buffer = piece::flip(game_state.piece_buffer, FlipDir::Vertical);
            }

            // Rotate pieces
            if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::PageUp) {
                use piece::RotateDir;
                game_state.piece_buffer = piece::rotate(game_state.piece_buffer, RotateDir::Left);
            }

            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::PageDown) {
                use piece::RotateDir;
                game_state.piece_buffer = piece::rotate(game_state.piece_buffer, RotateDir::Right);
            }

            let mouse_pos = Vec2::from(mouse_position());
            if is_mouse_button_pressed(MouseButton::Left) {
                if board_rect.contains(mouse_pos) {
                    // put a piece on the board
                    let (col, row) = (
                        ((mouse_pos.x - board_rect.x) / tile_size) as usize,
                        ((mouse_pos.y - board_rect.y) / tile_size) as usize,
                    );
                    dbg!(row, col);
                    game_state.try_advance_turn(row, col);
                } else if piece_rect.contains(mouse_pos) {
                    // Change selected piece.
                    let piece_size = 5. * ui_tile_size;
                    let (col, row) = (
                        ((mouse_pos.x - piece_rect.x) / piece_size) as usize,
                        ((mouse_pos.y - piece_rect.y) / piece_size) as usize,
                    );
                    dbg!(row, col);

                    let piece_id = row * 11 + col;
                    if game_state.players[game_state.current_player]
                        .remaining_pieces
                        .contains(piece_id)
                    {
                        game_state.selected_piece = Some(piece_id);
                        game_state.piece_buffer = piece::SHAPES[piece_id];
                    }
                } else {
                    game_state.selected_piece = None;
                    game_state.piece_buffer = piece::EMPTY_SHAPE;
                }
            }
        } // input section
        next_frame().await;
    }
}
