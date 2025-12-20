use tetris::commands::{CommandTable, TokenStream};
use tetris::config::parse_args;
use tetris::render_text::print_two_boards;
use tetris::game::Game;
use tetris::block::{BlockKind};

fn next_non_newline(ts: &mut TokenStream, pending: &mut Vec<String>) -> Option<String> {
    loop {
        let t = ts.next_token(pending)?;
        if t == "\n" { continue; }
        return Some(t);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cfg = parse_args(&args);

    let mut table = CommandTable::new();
    let mut stream = TokenStream::new();

    // Create game
    let mut game = match Game::new(cfg.seed, cfg.level, cfg.script_file1.clone(), cfg.script_file2.clone()) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Init error: {}", e);
            return;
        }
    };

    // Initial draw
    print_two_boards(&game.p1.grid, &game.p2.grid, game.p1.has_blind(), game.p2.has_blind(), game.system_hi);
    println!("Current player: {}", game.current_player);

    while game.running {
        let token = match next_non_newline(&mut stream, &mut table.pending) {
            Some(t) => t,
            None => break,
        };

        let (mut repeat, command) = match table.parse_command_token(&token) {
            Ok(x) => x,
            Err(msg) => {
                if !msg.is_empty() { eprintln!("{}", msg); }
                print_two_boards(&game.p1.grid, &game.p2.grid, game.p1.has_blind(), game.p2.has_blind(), game.system_hi);
                if game.running {
                    println!("Current player: {}", game.current_player);
                }
                continue;
            }
        };

        // Macro expansion if needed
        if table.is_macro_name(&command) {
            if let Some(seq) = table.macro_seq(&command) {
                if seq.is_empty() {
                    eprintln!("macro '{}' has empty body; ignoring.", command);
                } else {
                    for _ in 0..repeat {
                        for t in seq.iter().rev() {
                            table.pending.push(t.clone());
                        }
                    }
                }
            }
            continue;
        }

        let current_player = game.current_player;
        let heavy = if current_player == 1 { game.p1.level.is_heavy() } else { game.p2.level.is_heavy() };

        // Helper closure to access current player's mutable state safely
        // We'll do it with matches inside each command to avoid borrow conflicts.

        match command.as_str() {
            "quit" => break,

            "sequence" => {
                // ignore repeat (already forced to 1)
                let Some(file) = next_non_newline(&mut stream, &mut table.pending) else {
                    eprintln!("Missing file name for sequence");
                    continue;
                };
                if let Err(e) = stream.push_file(&file) {
                    eprintln!("{}", e);
                }
            }

            "norandom" => {
                let Some(file) = next_non_newline(&mut stream, &mut table.pending) else {
                    eprintln!("Missing file name for norandom");
                    continue;
                };

                let lvl = if current_player == 1 { game.p1.level.number() } else { game.p2.level.number() };
                if lvl < 3 {
                    eprintln!("norandom is only relevant in levels 3 and 4");
                } else {
                    let res = if current_player == 1 { game.p1.level.load_sequence(&file).and_then(|_| game.p1.level.set_random(false)) }
                              else { game.p2.level.load_sequence(&file).and_then(|_| game.p2.level.set_random(false)) };
                    if let Err(e) = res { eprintln!("norandom error: {}", e); }
                }
            }

            "random" => {
                let lvl = if current_player == 1 { game.p1.level.number() } else { game.p2.level.number() };
                if lvl < 3 {
                    eprintln!("random is only relevant in levels 3 and 4");
                } else {
                    let res = if current_player == 1 { game.p1.level.set_random(true) } else { game.p2.level.set_random(true) };
                    if let Err(e) = res { eprintln!("{}", e); }
                }
            }

            // Force current block to a specific type (I/J/L/S/T/O/Z)
            "I" | "J" | "L" | "S" | "T" | "O" | "Z" => {
                for _ in 0..repeat {
                    let kind = BlockKind::from_char(command.chars().next().unwrap()).unwrap();
                    let res = if current_player == 1 { game.p1.force_replace_current(kind) } else { game.p2.force_replace_current(kind) };
                    if let Err(e) = res {
                        eprintln!("Force-block error: {}", e);
                        println!("Game over, player {} lost.", current_player);
                        game.running = false;
                        break;
                    }
                }
            }

            "restart" => {
                if let Err(e) = game.restart() {
                    eprintln!("restart error: {}", e);
                    break;
                }
            }

            "cw" => {
                for _ in 0..repeat {
                    if !game.running { break; }
                    if current_player == 1 {
                        game.p1.cur.rotate_cw(&mut game.p1.grid);
                        if heavy {
                            if !game.p1.cur.move_down(&mut game.p1.grid) {
                                game.handle_landing(1);
                                if game.running {
                                    // heavy rotate can cause landing => post-drop
                                    game.p1.on_drop_effects();
                                    if game.running && game.p1.last_cleared >= 2 {
                                        // ask action
                                        handle_special_action(&mut game, &mut stream, &mut table, 1);
                                    }
                                    game.current_player = 2;
                                }
                                break;
                            }
                        }
                    } else {
                        game.p2.cur.rotate_cw(&mut game.p2.grid);
                        if heavy {
                            if !game.p2.cur.move_down(&mut game.p2.grid) {
                                game.handle_landing(2);
                                if game.running {
                                    game.p2.on_drop_effects();
                                    if game.running && game.p2.last_cleared >= 2 {
                                        handle_special_action(&mut game, &mut stream, &mut table, 2);
                                    }
                                    game.current_player = 1;
                                }
                                break;
                            }
                        }
                    }
                }
            }

            "ccw" => {
                for _ in 0..repeat {
                    if !game.running { break; }
                    if current_player == 1 {
                        game.p1.cur.rotate_ccw(&mut game.p1.grid);
                        if heavy {
                            if !game.p1.cur.move_down(&mut game.p1.grid) {
                                game.handle_landing(1);
                                if game.running {
                                    game.p1.on_drop_effects();
                                    if game.running && game.p1.last_cleared >= 2 {
                                        handle_special_action(&mut game, &mut stream, &mut table, 1);
                                    }
                                    game.current_player = 2;
                                }
                                break;
                            }
                        }
                    } else {
                        game.p2.cur.rotate_ccw(&mut game.p2.grid);
                        if heavy {
                            if !game.p2.cur.move_down(&mut game.p2.grid) {
                                game.handle_landing(2);
                                if game.running {
                                    game.p2.on_drop_effects();
                                    if game.running && game.p2.last_cleared >= 2 {
                                        handle_special_action(&mut game, &mut stream, &mut table, 2);
                                    }
                                    game.current_player = 1;
                                }
                                break;
                            }
                        }
                    }
                }
            }

            "left" => {
                for _ in 0..repeat {
                    if !game.running { break; }

                    let moved = if current_player == 1 { game.p1.cur.move_left(&mut game.p1.grid) } else { game.p2.cur.move_left(&mut game.p2.grid) };
                    if !moved { break; }

                    let extra = {
                        let lvl_heavy = if current_player == 1 { game.p1.level.is_heavy() } else { game.p2.level.is_heavy() };
                        let eff_heavy = if current_player == 1 { game.p1.extra_heavy_after_horizontal() } else { game.p2.extra_heavy_after_horizontal() };
                        (if lvl_heavy { 1 } else { 0 }) + eff_heavy
                    };

                    if extra > 0 {
                        let mut forced_drop = false;
                        for _ in 0..extra {
                            let ok = if current_player == 1 { game.p1.cur.move_down(&mut game.p1.grid) } else { game.p2.cur.move_down(&mut game.p2.grid) };
                            if !ok {
                                game.handle_landing(current_player);
                                forced_drop = true;
                                break;
                            }
                        }
                        if forced_drop && game.running {
                            if current_player == 1 {
                                game.p1.on_drop_effects();
                                if game.p1.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 1); }
                                game.current_player = 2;
                            } else {
                                game.p2.on_drop_effects();
                                if game.p2.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 2); }
                                game.current_player = 1;
                            }
                            break;
                        }
                    }
                }
            }

            "right" => {
                for _ in 0..repeat {
                    if !game.running { break; }

                    let moved = if current_player == 1 { game.p1.cur.move_right(&mut game.p1.grid) } else { game.p2.cur.move_right(&mut game.p2.grid) };
                    if !moved { break; }

                    let extra = {
                        let lvl_heavy = if current_player == 1 { game.p1.level.is_heavy() } else { game.p2.level.is_heavy() };
                        let eff_heavy = if current_player == 1 { game.p1.extra_heavy_after_horizontal() } else { game.p2.extra_heavy_after_horizontal() };
                        (if lvl_heavy { 1 } else { 0 }) + eff_heavy
                    };

                    if extra > 0 {
                        let mut forced_drop = false;
                        for _ in 0..extra {
                            let ok = if current_player == 1 { game.p1.cur.move_down(&mut game.p1.grid) } else { game.p2.cur.move_down(&mut game.p2.grid) };
                            if !ok {
                                game.handle_landing(current_player);
                                forced_drop = true;
                                break;
                            }
                        }
                        if forced_drop && game.running {
                            if current_player == 1 {
                                game.p1.on_drop_effects();
                                if game.p1.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 1); }
                                game.current_player = 2;
                            } else {
                                game.p2.on_drop_effects();
                                if game.p2.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 2); }
                                game.current_player = 1;
                            }
                            break;
                        }
                    }
                }
            }

            "down" => {
                for _ in 0..repeat {
                    if !game.running { break; }

                    if current_player == 1 {
                        let ok1 = game.p1.cur.move_down(&mut game.p1.grid);
                        if !ok1 {
                            if heavy {
                                game.handle_landing(1);
                                if game.running {
                                    game.p1.on_drop_effects();
                                    if game.p1.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 1); }
                                    game.current_player = 2;
                                }
                                break;
                            }
                        } else if heavy {
                            let ok2 = game.p1.cur.move_down(&mut game.p1.grid);
                            if !ok2 {
                                game.handle_landing(1);
                                if game.running {
                                    game.p1.on_drop_effects();
                                    if game.p1.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 1); }
                                    game.current_player = 2;
                                }
                                break;
                            }
                        }
                    } else {
                        let ok1 = game.p2.cur.move_down(&mut game.p2.grid);
                        if !ok1 {
                            if heavy {
                                game.handle_landing(2);
                                if game.running {
                                    game.p2.on_drop_effects();
                                    if game.p2.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 2); }
                                    game.current_player = 1;
                                }
                                break;
                            }
                        } else if heavy {
                            let ok2 = game.p2.cur.move_down(&mut game.p2.grid);
                            if !ok2 {
                                game.handle_landing(2);
                                if game.running {
                                    game.p2.on_drop_effects();
                                    if game.p2.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 2); }
                                    game.current_player = 1;
                                }
                                break;
                            }
                        }
                    }
                }
            }

            "drop" => {
                // repeat is always 1 (multiplier ignored), but keep structure
                if current_player == 1 {
                    game.p1.cur.drop(&mut game.p1.grid);
                    game.handle_landing(1);
                    if game.running {
                        game.p1.on_drop_effects();
                        if game.p1.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 1); }
                        game.current_player = 2;
                    }
                } else {
                    game.p2.cur.drop(&mut game.p2.grid);
                    game.handle_landing(2);
                    if game.running {
                        game.p2.on_drop_effects();
                        if game.p2.last_cleared >= 2 { handle_special_action(&mut game, &mut stream, &mut table, 2); }
                        game.current_player = 1;
                    }
                }
            }

            "leveldown" => {
                for _ in 0..repeat {
                    let lvl = if current_player == 1 { game.p1.level.number() } else { game.p2.level.number() };
                    if lvl <= 0 { break; }
                    let _ = game.set_level(current_player, lvl - 1);
                }
            }

            "levelup" => {
                for _ in 0..repeat {
                    let lvl = if current_player == 1 { game.p1.level.number() } else { game.p2.level.number() };
                    if lvl >= 4 { break; }
                    let _ = game.set_level(current_player, lvl + 1);
                }
            }

            "rename" => {
                let Some(new_name) = next_non_newline(&mut stream, &mut table.pending) else {
                    eprintln!("Usage: rename <newname> <existingcommand>");
                    continue;
                };
                let Some(old_name) = next_non_newline(&mut stream, &mut table.pending) else {
                    eprintln!("Usage: rename <newname> <existingcommand>");
                    continue;
                };

                match table.define_alias(&new_name, &old_name) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => eprintln!("{}", e),
                }
            }

            "macro" => {
                // macro <name> <rest-of-line...>
                let Some(macro_name) = next_non_newline(&mut stream, &mut table.pending) else {
                    eprintln!("Usage: macro <name> <sequence-of-commands>");
                    continue;
                };

                // collect tokens until newline sentinel
                let mut seq: Vec<String> = Vec::new();
                loop {
                    let t = stream.next_token(&mut table.pending);
                    match t {
                        None => break,
                        Some(x) => {
                            if x == "\n" { break; }
                            seq.push(x);
                        }
                    }
                }

                match table.define_macro(&macro_name, seq) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => eprintln!("{}", e),
                }
            }

            _ => {
                eprintln!("Invalid command");
            }
        }

        print_two_boards(&game.p1.grid, &game.p2.grid, game.p1.has_blind(), game.p2.has_blind(), game.system_hi);
        if game.running {
            println!("Current player: {}", game.current_player);
        }
    }
}

fn handle_special_action(game: &mut Game, stream: &mut TokenStream, table: &mut CommandTable, acting_player: i32) {
    println!("Player {}, choose special action (blind / heavy / force): ", acting_player);

    let action = match next_non_newline(stream, &mut table.pending) {
        Some(a) => a,
        None => {
            eprintln!("No special action given, skipping.");
            return;
        }
    };

    if action == "force" {
        let block = match next_non_newline(stream, &mut table.pending) {
            Some(b) => b,
            None => {
                eprintln!("force: missing block type (I/J/L/S/T/O/Z)");
                return;
            }
        };
        game.apply_special_action(acting_player, &action, Some(&block));
    } else {
        game.apply_special_action(acting_player, &action, None);
    }
}
