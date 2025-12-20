use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::block::{Block, BlockKind};
use crate::effects::Effect;
use crate::level::generate_level;
use crate::player::PlayerState;

pub struct Game {
    pub rng: StdRng,
    pub system_hi: i32,
    pub p1: PlayerState,
    pub p2: PlayerState,
    pub current_player: i32,
    pub running: bool,

    pub start_level: i32,
    pub script1: String,
    pub script2: String,
}

impl Game {
    pub fn new(seed: Option<u64>, start_level: i32, script1: String, script2: String) -> Result<Self, String> {
        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::seed_from_u64(12345),
        };

        let p1 = PlayerState::new(start_level, &script1, &mut rng)?;
        let p2 = PlayerState::new(start_level, &script2, &mut rng)?;

        Ok(Game {
            rng,
            system_hi: 0,
            p1,
            p2,
            current_player: 1,
            running: true,
            start_level,
            script1,
            script2,
        })
    }

    pub fn restart(&mut self) -> Result<(), String> {
        self.p1 = PlayerState::new(self.start_level, &self.script1, &mut self.rng)?;
        self.p2 = PlayerState::new(self.start_level, &self.script2, &mut self.rng)?;
        self.current_player = 1;
        self.running = true;
        Ok(())
    }

    pub fn apply_special_action(&mut self, acting_player: i32, action: &str, param: Option<&str>) {
        let (victim, victim_id) = if acting_player == 1 {
            (&mut self.p2, 2)
        } else {
            (&mut self.p1, 1)
        };

        match action {
            "blind" => {
                victim.effects.push(Effect::blind());
            }
            "heavy" => {
                victim.effects.push(Effect::heavy());
            }
            "force" => {
                let Some(p) = param else {
                    eprintln!("force: missing block type (I/J/L/S/T/O/Z)");
                    return;
                };
                let mut it = p.chars();
                let Some(t) = it.next() else {
                    eprintln!("force: missing block type");
                    return;
                };
                if it.next().is_some() {
                    eprintln!("force: block type must be a single character");
                    return;
                }
                if let Some(kind) = BlockKind::from_char(t) {
                    if kind == BlockKind::Star {
                        eprintln!("force: invalid block type '*'");
                        return;
                    }
                    if let Err(e) = victim.force_replace_current(kind) {
                        println!("Game over, player {} lost.", victim_id);
                        eprintln!("{}", e);
                        self.running = false;
                    }
                } else {
                    eprintln!("force: invalid block type '{}'", t);
                }
            }
            _ => {
                eprintln!("Unknown special action '{}', ignoring.", action);
            }
        }
    }

    pub fn handle_landing(&mut self, player_idx: i32) {
        if !self.running { return; }

        let (p, system_hi) = if player_idx == 1 {
            (&mut self.p1, &mut self.system_hi)
        } else {
            (&mut self.p2, &mut self.system_hi)
        };

        let mut block_loss: HashMap<i32, i32> = HashMap::new();
        let cleared = p.grid.check_and_clear(&mut block_loss);
        p.last_cleared = cleared;

        if cleared > 0 {
            let lvl = p.level.number();
            let delta = (cleared + lvl) * (cleared + lvl);
            p.grid.add_score(delta);
            *system_hi = (*system_hi).max(p.grid.cur_score());
        }

        p.apply_block_loss(&block_loss, system_hi);

        p.level.notify_rows_cleared(cleared);
        p.level.notify_block_placed();

        // Level 4: star drop every 5 blocks without a clear
        if p.level.should_drop_star() {
            let star_id = p.next_block_id;
            p.next_block_id += 1;

            let mut star = Block::new(BlockKind::Star, star_id);
            if star.can_spawn(&p.grid) {
                p.register_block(star.id, p.level.number(), star.cells.len());
                star.write_to_grid(&mut p.grid);
                star.drop(&mut p.grid);

                let mut star_loss: HashMap<i32, i32> = HashMap::new();
                let extra = p.grid.check_and_clear(&mut star_loss);
                if extra > 0 {
                    let lvl = p.level.number();
                    let delta = (extra + lvl) * (extra + lvl);
                    p.grid.add_score(delta);
                    *system_hi = (*system_hi).max(p.grid.cur_score());
                }
                p.apply_block_loss(&star_loss, system_hi);
                p.level.notify_rows_cleared(extra);
            }
        }

        // Spawn next falling block from stored next_kind
        let new_id = p.next_block_id;
        p.next_block_id += 1;

        let new_cur = Block::new(p.next_kind, new_id);
        if !new_cur.can_spawn(&p.grid) {
            println!("Game over, player {} lost.", player_idx);
            self.running = false;
            return;
        }

        p.cur = new_cur;
        p.register_block(p.cur.id, p.level.number(), p.cur.cells.len());
        p.cur.write_to_grid(&mut p.grid);

        // Generate and show next preview block
        let nk = p.level.advance_kind(&mut self.rng);
        p.next_kind = nk;
        p.refresh_preview();
    }

    pub fn set_level(&mut self, player_idx: i32, new_level: i32) -> Result<(), String> {
        let p = if player_idx == 1 { &mut self.p1 } else { &mut self.p2 };
        let lvl = new_level.clamp(0, 4);
        p.level = generate_level(lvl, &p.script_file)?;
        p.grid.set_level_digit(p.level.number());
        Ok(())
    }
}
