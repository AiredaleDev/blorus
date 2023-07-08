//! Blokus clone written in Rust.
//!
//! This is a board game from my childhood. It's also a nice excuse to get comfortable with using async/await semantics over the network.

use bit_set::BitSet;
use macroquad::prelude::*;

mod piece;

pub type PieceID = usize;

/// Denotes possible tile colors. Also used to denote player colors.
#[derive(Default, Debug, Clone, Copy)]
pub enum TileColor {
    Red,
    Yellow,
    Green,
    Blue,
    #[default]
    Empty,
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
            TileColor::Empty => BLANK,
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
/// Constructed on lobby creation.
#[derive(Debug)]
pub struct GameState {
    /// The current state of the board.
    board: [[TileColor; 20]; 20],
    /// Player data.
    players: Vec<Player>,
    /// Points to player whose turn it is.
    /// `0 <= current_player <= 3`
    current_player: usize,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: [[TileColor::default(); 20]; 20],
            players: Vec::with_capacity(4),
            current_player: 0,
        }
    }
}

impl GameState {
    pub fn place_piece(&mut self, row: usize, col: usize) {
        debug_assert!(!self.players.is_empty());
        let player = &self.players[self.current_player];

        // REFACTOR: Maybe move the offset code into this function.
        let Some((row, col)) =
            piece::check_bounds_and_recenter(player.piece_buffer, row as isize, col as isize) else { return; };
        dbg!(row, col);

        for (dr, r) in player.piece_buffer.iter().enumerate() {
            for dc in r.iter_ones() {
                // Sometimes I wish Rust allowed signed indices.
                let r_ind = (row + dr as isize) as usize;
                let c_ind = (col + dc as isize) as usize;
                self.board[r_ind][c_ind] = player.color;
            }
        }
    }

    /// Update the current player's piece buffer based on the result of `f`.
    /// `f` must accept a [`piece::Shape`] and return a [`piece::Shape`].
    pub fn alter_current_piece_buffer(&mut self, f: impl Fn(piece::Shape) -> piece::Shape) {
        // lambda-brained update semantics xd --
        // TODO: Maybe remove, this is kinda silly.
        // Also, no idea how it will perform
        self.players[self.current_player].piece_buffer =
            f(self.players[self.current_player].piece_buffer);
    }
}

#[macroquad::main("Blorus")]
async fn main() {
    // Determines where/how large the board should be.
    // Modify these to move or scale the board as a proportion of the screen.
    // The board automatically resizes itself with the window.
    const BOARD_SIZE: f32 = 0.5;
    const BOARD_HORIZ_OFFSET: f32 = 0.25;
    const BOARD_VERT_OFFSET: f32 = 0.25;

    let mut game_state = GameState::default();
    // test:
    /*
    for i in 0..20 {
        for j in 0..20 {
            game_state.board[i][j] = TileColor::ALL[i % 5];
        }
    }
    */
    game_state.players.push(Player::new(TileColor::Blue));
    game_state.players[game_state.current_player].piece_buffer = piece::PIECE_SHAPES[10];

    // Main loop
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
                    game_state.board[row][col].into(),
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
        if board_rect.contains(Vec2::new(mx, my)) && is_mouse_button_pressed(MouseButton::Left) {
            let (col, row) = (
                ((mx - board_rect.x) / tile_size) as usize,
                ((my - board_rect.y) / tile_size) as usize,
            );
            println!("I put the new forgis on the jeep: ({col}, {row})");
            game_state.place_piece(row, col);
        }

        next_frame().await;
    }
}
