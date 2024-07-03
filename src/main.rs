//! Blokus clone written in Rust.
//!
//! This is a board game from my childhood. It's also a nice excuse to get comfortable with
//! networked programming in an environment where the performance requirements aren't too stringent.

use macroquad::{
    audio::{load_sound, play_sound, PlaySoundParams},
    prelude::*,
};

mod debug;
mod logic;
mod net;
mod piece;

use logic::GameState;

// Modify these to move or scale the board as a proportion of the screen.
// The board automatically resizes itself with the window.
const BOARD_SIZE: f32 = 0.5;
const BOARD_HORIZ_OFFSET: f32 = 0.25;
const BOARD_VERT_OFFSET: f32 = 0.25;

#[macroquad::main("Blorus")]
async fn main() {
    let mut game_state = GameState::new(4);
    let mut placement_hint = None;
    let win_texture = Texture2D::from_file_with_format(include_bytes!("../assets/WIN.png"), None);

    // File I/O in Macroquad isn't *actually* async, unless you're in a browser.
    // TODO: Remove conditional compilation if this ever becomes async on all platforms.
    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    coroutines::start_coroutine(async move {
        match load_sound("assets/SneakySnitch.ogg").await {
            Ok(music) => play_sound(
                music,
                PlaySoundParams {
                    looped: true,
                    volume: 1.,
                },
            ),
            Err(e) => eprintln!("Failed to load epic music :( -- {e}"),
        }
    });

    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    {
        clear_background(BEIGE);
        draw_text(
            "Loading...",
            0.75 * screen_width(),
            0.9 * screen_height(),
            0.05 * screen_height(),
            BLACK,
        );
        next_frame().await;
        match load_sound("assets/SneakySnitch.ogg").await {
            Ok(music) => play_sound(
                music,
                PlaySoundParams {
                    looped: true,
                    volume: 1.,
                },
            ),
            Err(e) => eprintln!("Failed to load epic music :( -- {e}"),
        }
    }

    // =================
    //  -- Main loop --
    // =================

    while !game_state.is_game_over() {
        if !game_state.can_make_move() {
            game_state.current_player = (game_state.current_player + 1) % game_state.players.len();
            game_state.pass_counter += 1;
        }

        clear_background(BEIGE);

        let tile_size = screen_height() * 0.045 * BOARD_SIZE;
        // x = board_left's x coord, y = board_top's y coord
        let board_top_left = Vec2::new(
            screen_width() * BOARD_SIZE - screen_height() * BOARD_HORIZ_OFFSET,
            screen_height() * BOARD_VERT_OFFSET,
        );

        let play_area_top_left = Vec2::new(
            board_top_left.x + screen_height() * 0.05 * BOARD_SIZE,
            board_top_left.y + screen_height() * 0.05 * BOARD_SIZE,
        );

        // wanted to halve the area so I multiply the side length by sqrt(2)/2.
        let ui_tile_size = tile_size * 0.5 * 1.414;
        // each piece graphic is 5 UI tiles wide, and there are at most 11 per row.
        let avail_pieces = Vec2::new(
            0.5 * screen_width() - 5. * 5.5 * ui_tile_size,
            0.8 * screen_height(),
        );

        draw_game_screen(
            &game_state,
            &placement_hint,
            board_top_left,
            play_area_top_left,
            avail_pieces,
            tile_size,
            ui_tile_size,
        );

        handle_input(
            &mut game_state,
            &mut placement_hint,
            play_area_top_left,
            avail_pieces,
            tile_size,
            ui_tile_size,
        );

        next_frame().await;
    }

    // Game over screen
    loop {
        let draw_params = DrawTextureParams {
            dest_size: Some(Vec2::new(screen_width(), screen_height())),
            ..Default::default()
        };
        draw_texture_ex(win_texture, 0., 0., WHITE, draw_params);
        let winning_player = &game_state.players[game_state.current_player];
        draw_text(
            &format!("{:?}", winning_player.color),
            screen_width() / 2.,
            screen_height() / 2.,
            72.,
            winning_player.color.into(),
        );
        // TODO: *return* from this function instead so you can start a new lobby.
        next_frame().await;
    }
}

fn draw_game_screen(
    game_state: &GameState,
    placement_hint: &Option<UVec2>,
    // mayhaps I should bundle these together into "screeninfo"
    board_top_left: Vec2,
    play_area_top_left: Vec2,
    avail_pieces_top_left: Vec2,
    tile_size: f32,
    ui_tile_size: f32,
) {
    // Board
    draw_rectangle(
        board_top_left.x,
        board_top_left.y,
        screen_height() * BOARD_SIZE,
        screen_height() * BOARD_SIZE,
        GRAY,
    );

    // Draw the colorful tiles
    for row in 0..20 {
        for col in 0..20 {
            draw_rectangle(
                play_area_top_left.x + col as f32 * tile_size,
                play_area_top_left.y + row as f32 * tile_size,
                tile_size,
                tile_size,
                game_state.board[row + 1][col + 1].into(),
            );

            // Ah, that's right, I only expressed a desire to highlight the square
            // when I wanted the whole buffer to be drawn...
            // We'll need to read the buffer.
            if let Some(UVec2 { x: l_col, y: l_row }) = *placement_hint {
                draw_rectangle(
                    play_area_top_left.x + l_col as f32 * tile_size,
                    play_area_top_left.y + l_row as f32 * tile_size,
                    tile_size,
                    tile_size,
                    game_state.current_player().color.highlight_color(),
                );
            }
        }
    }

    // Board Border
    draw_rectangle_lines(
        board_top_left.x,
        board_top_left.y,
        screen_height() * BOARD_SIZE,
        screen_height() * BOARD_SIZE,
        4.,
        BLACK,
    );

    // Play area border
    draw_rectangle_lines(
        play_area_top_left.x,
        play_area_top_left.y,
        screen_height() * 0.9 * BOARD_SIZE,
        screen_height() * 0.9 * BOARD_SIZE,
        4.,
        BLACK,
    );

    // grid time
    // vertical lines:
    for i in 1..20 {
        let line_x = play_area_top_left.x + i as f32 * tile_size;
        draw_line(
            line_x,
            play_area_top_left.y,
            line_x,
            play_area_top_left.y + 20. * tile_size,
            2.,
            BLACK,
        );
    }

    // horizontal lines:
    for i in 1..20 {
        let line_y = play_area_top_left.y + i as f32 * tile_size;
        draw_line(
            play_area_top_left.x,
            line_y,
            play_area_top_left.x + 20. * tile_size,
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
                    tile as f32 * ui_tile_size + avail_pieces_top_left.x + offset * col as f32,
                    r_ind as f32 * ui_tile_size + avail_pieces_top_left.y + offset * row as f32,
                    ui_tile_size,
                    ui_tile_size,
                    player.color.into(),
                );

                draw_rectangle_lines(
                    tile as f32 * ui_tile_size + avail_pieces_top_left.x + offset * col as f32,
                    r_ind as f32 * ui_tile_size + avail_pieces_top_left.y + offset * row as f32,
                    ui_tile_size,
                    ui_tile_size,
                    2.,
                    BLACK,
                );
            }
        }
    }
}

fn handle_input(
    game_state: &mut GameState,
    placement_hint: &mut Option<UVec2>,
    play_area_top_left: Vec2,
    avail_pieces_pt: Vec2,
    tile_size: f32,
    ui_tile_size: f32,
) {
    // click detection rect
    let board_rect = Rect::new(
        play_area_top_left.x,
        play_area_top_left.y,
        20. * tile_size,
        20. * tile_size,
    );

    let piece_rect = Rect::new(
        avail_pieces_pt.x,
        avail_pieces_pt.y,
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
    // I will simply pull move-validation out to run on mouse hover,
    // and then just place the piece. `try_advance_turn` is now likely to go.
    //
    // Ahh, so the rub is simply that I'm now also going to need to remember
    // where I last highlighted. That's fine, just set aside some scratch space
    // local to the client. It doesn't make sense to send that over the network anyway.
    // I'm starting to think we're going to need a "UI state"

    if board_rect.contains(mouse_pos) {
        let corner = uvec2(
            ((mouse_pos.x - board_rect.x) / tile_size) as u32,
            ((mouse_pos.y - board_rect.y) / tile_size) as u32,
        );

        // This is a really leaky abstraction.
        // I'm feeling the assumptions I made earlier.
        if let Some((adj_row, adj_col)) = game_state.check_bounds_and_recenter(
            corner.y as isize,
            corner.x as isize,
        ) {
            // Why did I need to do "+1" here?
            *placement_hint = if game_state.valid_move(adj_row + 1, adj_col + 1) {
                Some(corner)
            } else {
                None
            };
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if board_rect.contains(mouse_pos) {
            // put a piece on the board
            let (col, row) = (
                ((mouse_pos.x - board_rect.x) / tile_size) as usize,
                ((mouse_pos.y - board_rect.y) / tile_size) as usize,
            );
            dbg!(row, col);
            
            if let Some(UVec2 { x, y }) = *placement_hint {
                // Need to recenter -- this coupling is undesirable
                let (adj_row, adj_col) = game_state.check_bounds_and_recenter(y as isize, x as isize).unwrap();
                game_state.place_piece(adj_row, adj_col);
                game_state.end_turn();
                *placement_hint = None;
            }
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
                game_state.select_piece(Some(piece_id));
            }
        } else {
            game_state.select_piece(None);
        }
    }
}
