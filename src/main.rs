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
    current_piece: piece::Shape,
}

impl Player {
    pub fn new(color: TileColor) -> Self {
        Self {
            color,
            remaining_pieces: BitSet::from_iter(0..=20),
            current_piece: piece::EMPTY_SHAPE,
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    board: [TileColor; 400],
    players: Vec<Player>,
    current_player: usize,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: [TileColor::default(); 400],
            players: Vec::with_capacity(4),
            current_player: 0,
        }
    }
}

#[macroquad::main("Blorus")]
async fn main() {
    let mut game_state = GameState::default();
    // test:
    for i in 0..400 {
        game_state.board[i] = TileColor::ALL[i % 5];
    }

    loop {
        clear_background(BEIGE);

        let (board_left_x, board_top_y) = (
            screen_width() * 0.5 - screen_height() * 0.25,
            screen_height() * 0.25,
        );

        // Board
        draw_rectangle(
            board_left_x,
            board_top_y,
            screen_height() * 0.5,
            screen_height() * 0.5,
            GRAY,
        );

        // Draw the colorful tiles
        let tile_size = screen_height() * 0.0225;
        let play_area_left_x = board_left_x + screen_height() * 0.025;
        let play_area_top_y = board_top_y + screen_height() * 0.025;

        for row in 0..20 {
            for col in 0..20 {
                draw_rectangle(
                    play_area_left_x + col as f32 * tile_size,
                    play_area_top_y + row as f32 * tile_size,
                    tile_size,
                    tile_size,
                    game_state.board[row * 20 + col].into(),
                );
            }
        }

        // Board Border
        draw_rectangle_lines(
            board_left_x,
            board_top_y,
            screen_height() * 0.5,
            screen_height() * 0.5,
            4.,
            BLACK,
        );

        // Play area border
        draw_rectangle_lines(
            play_area_left_x,
            play_area_top_y,
            screen_height() * 0.45,
            screen_height() * 0.45,
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

        next_frame().await;
    }
}
