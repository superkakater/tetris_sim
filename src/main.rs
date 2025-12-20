use tetris::config::parse_args;
use tetris::game::Game;
use tetris::grid::{COLS, PLAY_BOTTOM, PLAY_TOP};
use tetris::block::BlockKind;

use macroquad::prelude::*;

const CELL: f32 = 25.0;
const ROWS_PLAY: usize = 18; // rows 4..21 inclusive
const GAP: f32 = 80.0;
const LEFT_MARGIN: f32 = 30.0;
const TOP_MARGIN: f32 = 70.0;

const BOARD_W: f32 = COLS as f32 * CELL;
const BOARD_H: f32 = ROWS_PLAY as f32 * CELL;

const WINDOW_W: i32 = (2.0 * BOARD_W + GAP + 2.0 * LEFT_MARGIN) as i32;
const WINDOW_H: i32 = (BOARD_H + 260.0) as i32;

fn window_conf() -> Conf {
    Conf {
        window_title: "Tetris (Rust)".to_string(),
        window_width: WINDOW_W,
        window_height: WINDOW_H,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiMode {
    Playing,
    ChooseAction { acting_player: i32 },
    ChooseForce { acting_player: i32 },
}

fn in_blind_region(r: usize, c: usize) -> bool {
    let blind_row_start = 9usize;
    let blind_row_end = 18usize;
    let blind_col_start = 2usize;
    let blind_col_end = 8usize;
    r >= blind_row_start && r <= blind_row_end && c >= blind_col_start && c <= blind_col_end
}

fn color_for_char(ch: char) -> Color {
    match ch {
        'I' => Color::new(0.0, 0.8, 0.9, 1.0),   // cyan-ish
        'J' => Color::new(0.2, 0.4, 0.9, 1.0),   // blue
        'L' => Color::new(1.0, 0.55, 0.0, 1.0),  // orange
        'O' => Color::new(1.0, 0.9, 0.2, 1.0),   // yellow
        'S' => Color::new(0.2, 0.85, 0.2, 1.0),  // green
        'Z' => Color::new(0.9, 0.2, 0.2, 1.0),   // red
        'T' => Color::new(0.75, 0.2, 0.85, 1.0), // magenta
        '*' => Color::new(0.55, 0.27, 0.07, 1.0),// brown
        '-' => BLACK,
        ' ' => Color::new(0.96, 0.96, 0.96, 1.0),
        _ => WHITE,
    }
}

/// Compute phantom landing positions without mutating the block.
/// Mirrors your C++ logic: treat a cell as blocking if it's not empty AND not owned by this same block id.
fn compute_phantom_positions(
    block_cells: &[(i32, i32)],
    block_id: i32,
    grid: &tetris::grid::Grid,
) -> Vec<(i32, i32)> {
    let mut cells: Vec<(i32, i32)> = block_cells.to_vec();

    loop {
        let mut can_down = true;
        for &(r, c) in &cells {
            let nr = r + 1;
            if nr > PLAY_BOTTOM as i32 {
                can_down = false;
                break;
            }
            let ch = grid.get(nr as usize, c as usize);
            let bid = grid.get_block_id(nr as usize, c as usize);
            if ch != ' ' && bid != block_id {
                can_down = false;
                break;
            }
        }
        if !can_down {
            break;
        }
        for p in &mut cells {
            p.0 += 1;
        }
    }

    cells
}

fn draw_board(game: &Game, player: i32, offset_x: f32, offset_y: f32) {
    let (g, cur_block, blind, lvl, score) = if player == 1 {
        (
            &game.p1.grid,
            &game.p1.cur,
            game.p1.has_blind(),
            game.p1.level.number(),
            game.p1.grid.cur_score(),
        )
    } else {
        (
            &game.p2.grid,
            &game.p2.cur,
            game.p2.has_blind(),
            game.p2.level.number(),
            game.p2.grid.cur_score(),
        )
    };

    // Board background
    draw_rectangle(
        offset_x - 5.0,
        offset_y - 5.0,
        BOARD_W + 10.0,
        BOARD_H + 10.0,
        Color::new(1.0, 1.0, 1.0, 1.0),
    );

    // Top texts
    draw_text(&format!("Level: {}", lvl), offset_x, offset_y - 35.0, 24.0, BLACK);
    draw_text(&format!("Score: {}", score), offset_x, offset_y - 12.0, 24.0, BLACK);

    // Border frame
    draw_rectangle_lines(offset_x, offset_y, BOARD_W, BOARD_H, 2.0, BLACK);

    // Draw cells (matrix rows 4..21)
    let m = g.matrix();
    for r in PLAY_TOP..=PLAY_BOTTOM {
        for c in 0..COLS {
            let mut ch = m[r][c];
            let blind_cell = blind && in_blind_region(r, c);
            if blind_cell {
                ch = '?';
            }

            let mut col = if blind_cell { BLACK } else { color_for_char(ch) };
            if ch == '?' {
                col = BLACK;
            }

            let x = offset_x + c as f32 * CELL;
            let y = offset_y + (r - PLAY_TOP) as f32 * CELL;

            draw_rectangle(x + 1.0, y + 1.0, CELL - 2.0, CELL - 2.0, col);
        }
    }

    // Phantom block outline
    if game.running {
        let cells: Vec<(i32, i32)> = cur_block.cells.iter().map(|c| (c.r, c.c)).collect();
        let phantom = compute_phantom_positions(&cells, cur_block.id, g);

        for (r, c) in phantom {
            if g.get(r as usize, c as usize) != ' ' {
                continue;
            }
            let x = offset_x + c as f32 * CELL;
            let y = offset_y + (r as usize - PLAY_TOP) as f32 * CELL;

            draw_rectangle_lines(
                x + 2.0,
                y + 2.0,
                CELL - 4.0,
                CELL - 4.0,
                1.5,
                Color::new(0.2, 0.2, 0.2, 1.0),
            );
        }
    }

    // Next preview (matrix rows 24..25)
    let next_y = offset_y + BOARD_H + 35.0;
    draw_text("Next:", offset_x, next_y, 24.0, BLACK);

    let mini = CELL * 0.75;
    let mini_y0 = next_y + 10.0;
    for rr in 24..=25 {
        for c in 0..COLS {
            let ch = m[rr][c];
            if ch == ' ' {
                continue;
            }
            let col = color_for_char(ch);
            let x = offset_x + c as f32 * mini;
            let y = mini_y0 + (rr - 23) as f32 * mini;
            draw_rectangle(x + 1.0, y + 1.0, mini - 2.0, mini - 2.0, col);
        }
    }
}

fn key_to_block_kind() -> Option<BlockKind> {
    if is_key_pressed(KeyCode::I) {
        Some(BlockKind::I)
    } else if is_key_pressed(KeyCode::J) {
        Some(BlockKind::J)
    } else if is_key_pressed(KeyCode::L) {
        Some(BlockKind::L)
    } else if is_key_pressed(KeyCode::S) {
        Some(BlockKind::S)
    } else if is_key_pressed(KeyCode::T) {
        Some(BlockKind::T)
    } else if is_key_pressed(KeyCode::O) {
        Some(BlockKind::O)
    } else if is_key_pressed(KeyCode::Z) {
        Some(BlockKind::Z)
    } else {
        None
    }
}

fn end_turn_or_prompt_special(game: &mut Game, ui: &mut UiMode, acting_player: i32) {
    if acting_player == 1 {
        game.p1.on_drop_effects();
        if game.running && game.p1.last_cleared >= 2 {
            *ui = UiMode::ChooseAction { acting_player };
            return;
        }
        game.current_player = 2;
    } else {
        game.p2.on_drop_effects();
        if game.running && game.p2.last_cleared >= 2 {
            *ui = UiMode::ChooseAction { acting_player };
            return;
        }
        game.current_player = 1;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cfg = parse_args(&args);

    let mut game = match Game::new(cfg.seed, cfg.level, cfg.script_file1.clone(), cfg.script_file2.clone()) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Init error: {}", e);
            return;
        }
    };

    let mut ui = UiMode::Playing;

    loop {
        clear_background(Color::new(0.97, 0.97, 0.97, 1.0));

        // Header
        let hi = format!("Hi Score: {}", game.system_hi);
        let dim = measure_text(&hi, None, 28, 1.0);
        draw_text(&hi, (WINDOW_W as f32 - dim.width) * 0.5, 24.0, 28.0, BLACK);

        // Board positions
        let b1x = LEFT_MARGIN;
        let b2x = LEFT_MARGIN + BOARD_W + GAP;

        draw_board(&game, 1, b1x, TOP_MARGIN);
        draw_board(&game, 2, b2x, TOP_MARGIN);

        // Footer
        let cp = format!("Current player: {}", game.current_player);
        draw_text(&cp, LEFT_MARGIN, WINDOW_H as f32 - 18.0, 24.0, BLACK);

        // Overlays
        match ui {
            UiMode::Playing => {}
            UiMode::ChooseAction { acting_player } => {
                let msg = format!(
                    "Player {}, choose special action: [B]lind / [H]eavy / [F]orce",
                    acting_player
                );
                draw_rectangle(
                    20.0,
                    WINDOW_H as f32 - 90.0,
                    WINDOW_W as f32 - 40.0,
                    60.0,
                    Color::new(1.0, 1.0, 1.0, 0.92),
                );
                draw_rectangle_lines(
                    20.0,
                    WINDOW_H as f32 - 90.0,
                    WINDOW_W as f32 - 40.0,
                    60.0,
                    2.0,
                    BLACK,
                );
                draw_text(&msg, 30.0, WINDOW_H as f32 - 52.0, 24.0, BLACK);
            }
            UiMode::ChooseForce { acting_player } => {
                let msg = format!("Player {}, force block: press I/J/L/S/T/O/Z", acting_player);
                draw_rectangle(
                    20.0,
                    WINDOW_H as f32 - 90.0,
                    WINDOW_W as f32 - 40.0,
                    60.0,
                    Color::new(1.0, 1.0, 1.0, 0.92),
                );
                draw_rectangle_lines(
                    20.0,
                    WINDOW_H as f32 - 90.0,
                    WINDOW_W as f32 - 40.0,
                    60.0,
                    2.0,
                    BLACK,
                );
                draw_text(&msg, 30.0, WINDOW_H as f32 - 52.0, 24.0, BLACK);
            }
        }

        // Game over overlay
        if !game.running {
            let msg = "Game Over — press R to restart, Esc to quit";
            draw_rectangle(20.0, 35.0, WINDOW_W as f32 - 40.0, 50.0, Color::new(1.0, 0.95, 0.95, 0.95));
            draw_rectangle_lines(20.0, 35.0, WINDOW_W as f32 - 40.0, 50.0, 2.0, RED);
            draw_text(msg, 30.0, 68.0, 24.0, BLACK);

            if is_key_pressed(KeyCode::R) {
                if let Err(e) = game.restart() {
                    eprintln!("restart error: {}", e);
                }
                ui = UiMode::Playing;
            }
            if is_key_pressed(KeyCode::Escape) {
                break;
            }

            next_frame().await;
            continue;
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Special action modes
        match ui {
            UiMode::ChooseAction { acting_player } => {
                if is_key_pressed(KeyCode::B) {
                    game.apply_special_action(acting_player, "blind", None);
                    ui = UiMode::Playing;
                    game.current_player = if acting_player == 1 { 2 } else { 1 };
                } else if is_key_pressed(KeyCode::H) {
                    game.apply_special_action(acting_player, "heavy", None);
                    ui = UiMode::Playing;
                    game.current_player = if acting_player == 1 { 2 } else { 1 };
                } else if is_key_pressed(KeyCode::F) {
                    ui = UiMode::ChooseForce { acting_player };
                }
                next_frame().await;
                continue;
            }
            UiMode::ChooseForce { acting_player } => {
                if let Some(kind) = key_to_block_kind() {
                    let param = kind.to_char().to_string();
                    game.apply_special_action(acting_player, "force", Some(&param));
                    ui = UiMode::Playing;
                    if game.running {
                        game.current_player = if acting_player == 1 { 2 } else { 1 };
                    }
                }
                next_frame().await;
                continue;
            }
            UiMode::Playing => {}
        }

        // Restart
        if is_key_pressed(KeyCode::R) {
            if let Err(e) = game.restart() {
                eprintln!("restart error: {}", e);
            }
            ui = UiMode::Playing;
            next_frame().await;
            continue;
        }

        // Level up/down
        if is_key_pressed(KeyCode::PageUp) {
            let lvl = if game.current_player == 1 { game.p1.level.number() } else { game.p2.level.number() };
            let _ = game.set_level(game.current_player, (lvl + 1).min(4));
        }
        if is_key_pressed(KeyCode::PageDown) {
            let lvl = if game.current_player == 1 { game.p1.level.number() } else { game.p2.level.number() };
            let _ = game.set_level(game.current_player, (lvl - 1).max(0));
        }

        // Keyboard commands
        let left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        let right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        let cw = is_key_pressed(KeyCode::E);
        let ccw = is_key_pressed(KeyCode::Q);
        let drop = is_key_pressed(KeyCode::Space);

        let acting_player = game.current_player;

        if left {
            let moved = if acting_player == 1 {
                game.p1.cur.move_left(&mut game.p1.grid)
            } else {
                game.p2.cur.move_left(&mut game.p2.grid)
            };
            if moved {
                let extra = if acting_player == 1 {
                    (if game.p1.level.is_heavy() { 1 } else { 0 }) + game.p1.extra_heavy_after_horizontal()
                } else {
                    (if game.p2.level.is_heavy() { 1 } else { 0 }) + game.p2.extra_heavy_after_horizontal()
                };
                let mut forced = false;
                for _ in 0..extra {
                    let ok = if acting_player == 1 {
                        game.p1.cur.move_down(&mut game.p1.grid)
                    } else {
                        game.p2.cur.move_down(&mut game.p2.grid)
                    };
                    if !ok {
                        game.handle_landing(acting_player);
                        forced = true;
                        break;
                    }
                }
                if forced && game.running {
                    end_turn_or_prompt_special(&mut game, &mut ui, acting_player);
                }
            }
        } else if right {
            let moved = if acting_player == 1 {
                game.p1.cur.move_right(&mut game.p1.grid)
            } else {
                game.p2.cur.move_right(&mut game.p2.grid)
            };
            if moved {
                let extra = if acting_player == 1 {
                    (if game.p1.level.is_heavy() { 1 } else { 0 }) + game.p1.extra_heavy_after_horizontal()
                } else {
                    (if game.p2.level.is_heavy() { 1 } else { 0 }) + game.p2.extra_heavy_after_horizontal()
                };
                let mut forced = false;
                for _ in 0..extra {
                    let ok = if acting_player == 1 {
                        game.p1.cur.move_down(&mut game.p1.grid)
                    } else {
                        game.p2.cur.move_down(&mut game.p2.grid)
                    };
                    if !ok {
                        game.handle_landing(acting_player);
                        forced = true;
                        break;
                    }
                }
                if forced && game.running {
                    end_turn_or_prompt_special(&mut game, &mut ui, acting_player);
                }
            }
        } else if down {
            if acting_player == 1 {
                let ok1 = game.p1.cur.move_down(&mut game.p1.grid);
                if !ok1 {
                    if game.p1.level.is_heavy() {
                        game.handle_landing(1);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 1); }
                    }
                } else if game.p1.level.is_heavy() {
                    let ok2 = game.p1.cur.move_down(&mut game.p1.grid);
                    if !ok2 {
                        game.handle_landing(1);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 1); }
                    }
                }
            } else {
                let ok1 = game.p2.cur.move_down(&mut game.p2.grid);
                if !ok1 {
                    if game.p2.level.is_heavy() {
                        game.handle_landing(2);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 2); }
                    }
                } else if game.p2.level.is_heavy() {
                    let ok2 = game.p2.cur.move_down(&mut game.p2.grid);
                    if !ok2 {
                        game.handle_landing(2);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 2); }
                    }
                }
            }
        } else if cw {
            if acting_player == 1 {
                game.p1.cur.rotate_cw(&mut game.p1.grid);
                if game.p1.level.is_heavy() {
                    if !game.p1.cur.move_down(&mut game.p1.grid) {
                        game.handle_landing(1);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 1); }
                    }
                }
            } else {
                game.p2.cur.rotate_cw(&mut game.p2.grid);
                if game.p2.level.is_heavy() {
                    if !game.p2.cur.move_down(&mut game.p2.grid) {
                        game.handle_landing(2);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 2); }
                    }
                }
            }
        } else if ccw {
            if acting_player == 1 {
                game.p1.cur.rotate_ccw(&mut game.p1.grid);
                if game.p1.level.is_heavy() {
                    if !game.p1.cur.move_down(&mut game.p1.grid) {
                        game.handle_landing(1);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 1); }
                    }
                }
            } else {
                game.p2.cur.rotate_ccw(&mut game.p2.grid);
                if game.p2.level.is_heavy() {
                    if !game.p2.cur.move_down(&mut game.p2.grid) {
                        game.handle_landing(2);
                        if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 2); }
                    }
                }
            }
        } else if drop {
            if acting_player == 1 {
                game.p1.cur.drop(&mut game.p1.grid);
                game.handle_landing(1);
                if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 1); }
            } else {
                game.p2.cur.drop(&mut game.p2.grid);
                game.handle_landing(2);
                if game.running { end_turn_or_prompt_special(&mut game, &mut ui, 2); }
            }
        }

        next_frame().await;
    }
}
