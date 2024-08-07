//! Blokus clone written in Rust.
//!
//! This is a board game from my childhood. It's also a nice excuse to get comfortable with
//! networked programming in an environment where the performance requirements aren't too stringent.

use macroquad::{
    audio::{load_sound, play_sound, PlaySoundParams},
    prelude::*,
    ui::{root_ui, widgets::Button},
};
use smallvec::{SmallVec, ToSmallVec};
use std::env::args;

mod debug;
mod logic;
mod net;
mod piece;

use logic::{GameState, Player, TileColor};

// Modify these to move or scale the board as a proportion of the screen.
// The board automatically resizes itself with the window.
const BOARD_SIZE: f32 = 0.5;
const BOARD_HORIZ_OFFSET: f32 = 0.25;
const BOARD_VERT_OFFSET: f32 = 0.25;

#[macroquad::main("Blorus")]
async fn main() {
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

    let mut args = args().skip(1);
    // I feel like I should've been able to pattern-match this.
    // Anyway, I left this branch in so I could still play the game quickly.
    if let Some(demo_flag) = args.next() {
        if demo_flag == "demo" {
            let players = TileColor::DEFAULT_ORDER.map(Player::new);
            game_loop(players.into()).await;
        }
    } else {
        setup_screen().await;
    }
}

/// Local multiplayer setup screen
async fn setup_screen() {
    let mut players = Player::default_order(2);
    // Change to "while not (exit condition)"
    loop {
        let mut dropped_players = SmallVec::<[usize; 4]>::new();

        clear_background(BEIGE);
        // You know, maybe it would be fun for networked multiplayer to let you fidget
        // with the piece that represents you and have it display to everyone in the lobby.
        // That's a good sort of thing to have if you're waiting on someone to arrive.

        // So this screen just has an "add/remove player" button, a "change color" button for each player,
        // as well as a play/ready button. The UI is almost 100% repurposable for both local and
        // online multiplayer.

        // Forcing people to leave and rejoin to get their desired color is annoying, so there will
        // also be a "swap color" button to offer to switch colors with someone else.
        let player_status_dims = vec2(screen_width() / 16., screen_height() / 8.);
        let player_status_padding = screen_height() / 16.;
        let plen_f = players.len() as f32;

        let player_status_region_dims = vec2(
            player_status_dims.x * plen_f + player_status_padding * (plen_f + 1.),
            player_status_dims.y + player_status_padding * 2.,
        );
        let player_status_region_pos = centered_at(
            vec2(screen_width() / 2., screen_height() / 2.),
            player_status_region_dims,
        );
        let tile_size = 0.1 * player_status_region_dims.y;
        for (i, p) in players.iter().enumerate() {
            let elem_x = player_status_dims.x * i as f32 + player_status_padding * (i + 1) as f32;
            // Now, each player gets drawn here.
            let player_repr = piece::SHAPES[17 + i];
            draw_piece(
                player_repr,
                p.color,
                player_status_region_pos + vec2(elem_x, 0.),
                tile_size,
                true,
            );

            // Under each player there will be a "drop",
            // "change color", and "swap color" button.
            let drop_button = Button::new("Drop out")
                .position(player_status_region_pos + vec2(elem_x, 5. * tile_size));
            if drop_button.ui(&mut root_ui()) {
                dropped_players.push(i);
            }
        }

        // We defer dropping the players until now since Rust understandably
        // diallows modifying a collection while iterating it.
        for p_ind in dropped_players {
            players.remove(p_ind);
        }

        // We now also place the "Add Player" button below.
        let player_button_dims = medium_ui_button_dims();
        let player_button_pos = centered_at(
            vec2(screen_width() / 2., screen_height() * 0.75),
            player_button_dims,
        );
        let add_player_button = Button::new("Add player")
            .position(player_button_pos)
            .size(player_button_dims);
        if add_player_button.ui(&mut root_ui()) {
            // Okay, let's add a player, first color not already in the list.
            if players.len() < 4 {
                if let Some(color) = TileColor::DEFAULT_ORDER
                    .into_iter()
                    .find(|c| players.iter().all(|p| p.color != *c))
                {
                    players.push(Player::new(color));
                }
            }
        }

        let start_game_button = Button::new("Begin!")
            .position(
                player_button_pos
                    + vec2(0., 1. / 16. * screen_height() + medium_ui_button_padding()),
            )
            .size(player_button_dims);
        if start_game_button.ui(&mut root_ui()) {
            // Player data is just two integers, pretty cheap to copy.
            // I wonder why BitSets do not implement `Copy`. They should just be
            // integers, unlike BitVecs which have a notion of "push/pop".
            game_loop(players.clone()).await;
        }

        next_frame().await;
    }
}

async fn game_loop(players: SmallVec<[Player; 4]>) {
    let mut game_state = GameState::with_players(players);
    // TODO: Put this somewhere more sane -- it now has the final say on whether or not the player
    // is making a valid move!
    let mut placement_hint = None;
    let win_texture = Texture2D::from_file_with_format(include_bytes!("../assets/WIN.png"), None);

    while !game_state.is_game_over() {
        if !game_state.can_make_move() {
            game_state.end_turn();
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
        let play_again_dims = medium_ui_button_dims();
        let play_again_pos = centered_at(
            vec2(screen_width() / 2., 0.75 * screen_height()),
            play_again_dims,
        );
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

        let play_again_button = Button::new("Return to lobby")
            .position(play_again_pos)
            .size(play_again_dims);

        if play_again_button.ui(&mut root_ui()) {
            break;
        }

        next_frame().await;
    }
}

fn draw_game_screen(
    game_state: &GameState,
    placement_hint: &Option<IVec2>,
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
        }
    }

    if let Some(IVec2 { x: l_col, y: l_row }) = *placement_hint {
        // Okay, looks like we have a placeable piece -- let's draw where it could be.
        for (dr, r) in game_state.piece_buffer.iter().enumerate() {
            for dc in r.iter_ones() {
                let t_row = l_row + dr as i32;
                let t_col = l_col + dc as i32;
                draw_rectangle(
                    play_area_top_left.x + t_col as f32 * tile_size,
                    play_area_top_left.y + t_row as f32 * tile_size,
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
    let piece_left = 0.05 * screen_width();
    let piece_top = 0.35 * screen_height();
    if game_state.selected_piece.is_some() {
        // piece preview border
        draw_rectangle(
            piece_left - tile_size,
            piece_top - tile_size,
            7. * tile_size,
            7. * tile_size,
            GRAY,
        );

        draw_rectangle_lines(
            piece_left - tile_size,
            piece_top - tile_size,
            7. * tile_size,
            7. * tile_size,
            4.,
            BLACK,
        );

        // Piece preview
        draw_piece(
            game_state.piece_buffer,
            player.color,
            vec2(piece_left, piece_top),
            tile_size,
            true,
        );
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
    placement_hint: &mut Option<IVec2>,
    play_area_top_left: Vec2,
    avail_pieces_pt: Vec2,
    tile_size: f32,
    ui_tile_size: f32,
) {
    // click detection rects
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
        // Can't quite do `Option::map` since `update_suggestion` is T -> Option<U> not T -> U.
        *placement_hint = match *placement_hint {
            Some(proposed) => update_suggestion(&game_state, proposed),
            None => None,
        };
    }

    if [KeyCode::W, KeyCode::S, KeyCode::Up, KeyCode::Down]
        .into_iter()
        .any(is_key_pressed)
    {
        use piece::FlipDir;
        game_state.piece_buffer = piece::flip(game_state.piece_buffer, FlipDir::Vertical);
        *placement_hint = match *placement_hint {
            Some(proposed) => update_suggestion(&game_state, proposed),
            None => None,
        };
    }

    // Rotate pieces
    if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::PageUp) {
        use piece::RotateDir;
        game_state.piece_buffer = piece::rotate(game_state.piece_buffer, RotateDir::Left);
        *placement_hint = match *placement_hint {
            Some(proposed) => update_suggestion(&game_state, proposed),
            None => None,
        };
    }

    if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::PageDown) {
        use piece::RotateDir;
        game_state.piece_buffer = piece::rotate(game_state.piece_buffer, RotateDir::Right);
        *placement_hint = match *placement_hint {
            Some(proposed) => update_suggestion(&game_state, proposed),
            None => None,
        };
    }

    let mouse_pos = Vec2::from(mouse_position());
    // If this move is valid, mark it as such for the drawing logic.
    if board_rect.contains(mouse_pos) {
        let center = ivec2(
            ((mouse_pos.x - board_rect.x) / tile_size) as i32,
            ((mouse_pos.y - board_rect.y) / tile_size) as i32,
        );

        *placement_hint = update_suggestion(&game_state, center);
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if board_rect.contains(mouse_pos) {
            // put a piece on the board -- we know where, since we already validated!
            if let Some(corner) = *placement_hint {
                game_state.place_piece(corner);
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
            if game_state
                .current_player()
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

/// Given the center position and size of a UI element, return the position
/// of its topleft corner.
fn centered_at(center: Vec2, dims: Vec2) -> Vec2 {
    center - 0.5 * dims
}

fn medium_ui_button_dims() -> Vec2 {
    vec2(screen_height() / 4., screen_height() / 16.)
}

fn medium_ui_button_padding() -> f32 {
    1. / 64. * screen_height()
}

/// Updates the coordinates for the potential next move.
fn update_suggestion(game_state: &GameState, proposed: IVec2) -> Option<IVec2> {
    if let Some(corner) = game_state.check_bounds_and_recenter(proposed) {
        // Why did I need to do "+1" here?
        // I completely forgot what madness led me here.
        if game_state.valid_move(corner + IVec2::ONE) {
            Some(corner)
        } else {
            None
        }
    } else {
        None
    }
}

fn draw_piece(shape: piece::Shape, color: TileColor, at: Vec2, tile_size: f32, with_borders: bool) {
    // piece preview
    for (r_ind, row) in shape.iter().enumerate() {
        for tile in row.iter_ones() {
            draw_rectangle(
                tile as f32 * tile_size + at.x,
                r_ind as f32 * tile_size + at.y,
                tile_size,
                tile_size,
                color.into(),
            );

            if with_borders {
                draw_rectangle_lines(
                    tile as f32 * tile_size + at.x,
                    r_ind as f32 * tile_size + at.y,
                    tile_size,
                    tile_size,
                    2.,
                    BLACK,
                );
            }
        }
    }
}
