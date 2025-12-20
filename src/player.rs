use std::collections::HashMap;

use crate::block::{Block, BlockKind};
use crate::effects::Effect;
use crate::grid::Grid;
use crate::level::{generate_level, Level};

#[derive(Clone)]
pub struct BlockInfo {
    pub origin_level: i32,
    pub cells_remaining: i32,
}

pub struct PlayerState {
    pub grid: Grid,
    pub level: Level,
    pub cur: Block,
    pub next_kind: BlockKind,
    pub next_preview: Block, // id=0
    pub script_file: String,
    pub start_level: i32,

    pub effects: Vec<Effect>,
    pub last_cleared: i32,

    pub registry: HashMap<i32, BlockInfo>,
    pub next_block_id: i32,
}

impl PlayerState {
    pub fn new(start_level: i32, script_file: &str, rng: &mut rand::rngs::StdRng) -> Result<Self, String> {
        let mut grid = Grid::new();
        let mut level = generate_level(start_level, script_file)?;
        grid.set_level_digit(level.number());

        let cur_kind = level.peek_kind(rng);
        let next_kind = level.advance_kind(rng);

        let mut next_block_id = 1;
        let cur_id = next_block_id;
        next_block_id += 1;

        let cur = Block::new(cur_kind, cur_id);
        if !cur.can_spawn(&grid) {
            return Err("Game over on init: cannot place initial block".to_string());
        }

        let next_preview = Block::new(next_kind, 0);

        let mut p = PlayerState {
            grid,
            level,
            cur,
            next_kind,
            next_preview,
            script_file: script_file.to_string(),
            start_level,
            effects: Vec::new(),
            last_cleared: 0,
            registry: HashMap::new(),
            next_block_id,
        };

        p.register_block(p.cur.id, p.level.number(), p.cur.cells.len());
        p.cur.write_to_grid(&mut p.grid);
        p.refresh_preview();

        Ok(p)
    }

    pub fn has_blind(&self) -> bool {
        self.effects.iter().any(|e| e.is_blind())
    }

    pub fn extra_heavy_after_horizontal(&self) -> i32 {
        self.effects.iter().filter(|e| e.adds_heavy_on_horizontal()).count() as i32 * 2
    }

    pub fn on_drop_effects(&mut self) {
        for e in &mut self.effects {
            e.on_drop();
        }
        self.effects.retain(|e| !e.is_expired());
    }

    pub fn register_block(&mut self, block_id: i32, origin_level: i32, cells: usize) {
        if block_id <= 0 { return; }
        self.registry.insert(block_id, BlockInfo {
            origin_level,
            cells_remaining: cells as i32,
        });
    }

    pub fn apply_block_loss(&mut self, loss: &HashMap<i32, i32>, system_hi: &mut i32) {
        for (&bid, &lost) in loss {
            if bid == self.cur.id { continue; } // ignore active falling block (matches your C++)
            if let Some(info) = self.registry.get_mut(&bid) {
                info.cells_remaining -= lost;
                if info.cells_remaining <= 0 {
                    let lvl = info.origin_level;
                    let bonus = (lvl + 1) * (lvl + 1);
                    self.grid.add_score(bonus);
                    *system_hi = (*system_hi).max(self.grid.cur_score());
                    self.registry.remove(&bid);
                }
            }
        }
    }

    pub fn refresh_preview(&mut self) {
        self.next_preview = Block::new(self.next_kind, 0);
        let preview_cells: Vec<(i32, i32, char)> = self.next_preview.cells.iter()
            .map(|c| (c.r, c.c, c.ch))
            .collect();
        self.grid.show_next(&preview_cells);
    }

    pub fn force_replace_current(&mut self, kind: BlockKind) -> Result<(), String> {
        // Clear old falling block cells from grid
        let old_id = self.cur.id;
        self.cur.clear_from_grid(&mut self.grid);

        // Remove from registry with NO bonus (matches your C++)
        self.registry.remove(&old_id);

        // Create new block with new id
        let new_id = self.next_block_id;
        self.next_block_id += 1;

        let new_block = Block::new(kind, new_id);
        if !new_block.can_spawn(&self.grid) {
            return Err("Forced block cannot be placed; game over".to_string());
        }

        self.cur = new_block;
        self.register_block(self.cur.id, self.level.number(), self.cur.cells.len());
        self.cur.write_to_grid(&mut self.grid);

        Ok(())
    }
}
