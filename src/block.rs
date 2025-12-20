use crate::grid::{Grid, COLS, PLAY_BOTTOM, PLAY_TOP};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    T, S, Z, I, J, L, O, Star,
}

impl BlockKind {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            'T' => Some(BlockKind::T),
            'S' => Some(BlockKind::S),
            'Z' => Some(BlockKind::Z),
            'I' => Some(BlockKind::I),
            'J' => Some(BlockKind::J),
            'L' => Some(BlockKind::L),
            'O' => Some(BlockKind::O),
            '*' => Some(BlockKind::Star),
            _ => None,
        }
    }
    pub fn to_char(self) -> char {
        match self {
            BlockKind::T => 'T',
            BlockKind::S => 'S',
            BlockKind::Z => 'Z',
            BlockKind::I => 'I',
            BlockKind::J => 'J',
            BlockKind::L => 'L',
            BlockKind::O => 'O',
            BlockKind::Star => '*',
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub r: i32,
    pub c: i32,
    pub ch: char,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub kind: BlockKind,
    pub id: i32,          // >0 for tracked blocks, 0 for preview blocks
    pub cells: Vec<Cell>,
}

impl Block {
    pub fn new(kind: BlockKind, id: i32) -> Self {
        Self { kind, id, cells: spawn_cells(kind) }
    }

    pub fn write_to_grid(&self, g: &mut Grid) {
        for cell in &self.cells {
            g.set_cell(cell.r as usize, cell.c as usize, cell.ch, self.id);
        }
    }

    pub fn clear_from_grid(&self, g: &mut Grid) {
        for cell in &self.cells {
            g.clear_cell(cell.r as usize, cell.c as usize);
        }
    }

    pub fn can_spawn(&self, g: &Grid) -> bool {
        for cell in &self.cells {
            if cell.c < 0 || cell.c >= COLS as i32 { return false; }
            if cell.r < 0 { return false; }
            let r = cell.r as usize;
            let c = cell.c as usize;
            if r >= g.rows_total() { return false; }
            if g.get(r, c) != ' ' { return false; }
        }
        true
    }

    fn can_place(next: &[Cell], g: &Grid, self_id: i32) -> bool {
        for cell in next {
            if cell.c < 0 || cell.c >= COLS as i32 { return false; }
            if cell.r < PLAY_TOP as i32 || cell.r > PLAY_BOTTOM as i32 { return false; }
            let r = cell.r as usize;
            let c = cell.c as usize;
            let ch = g.get(r, c);
            let bid = g.get_block_id(r, c);
            if ch != ' ' && bid != self_id {
                return false;
            }
        }
        true
    }

    pub fn move_down(&mut self, g: &mut Grid) -> bool {
        self.clear_from_grid(g);

        let mut next = self.cells.clone();
        for c in &mut next { c.r += 1; }

        if next.iter().any(|c| c.r > PLAY_BOTTOM as i32) || !Self::can_place(&next, g, self.id) {
            self.write_to_grid(g);
            return false;
        }

        self.cells = next;
        self.write_to_grid(g);
        true
    }

    pub fn move_left(&mut self, g: &mut Grid) -> bool {
        self.clear_from_grid(g);

        let mut next = self.cells.clone();
        for c in &mut next { c.c -= 1; }

        if next.iter().any(|c| c.c < 0) || !Self::can_place(&next, g, self.id) {
            self.write_to_grid(g);
            return false;
        }

        self.cells = next;
        self.write_to_grid(g);
        true
    }

    pub fn move_right(&mut self, g: &mut Grid) -> bool {
        self.clear_from_grid(g);

        let mut next = self.cells.clone();
        for c in &mut next { c.c += 1; }

        if next.iter().any(|c| c.c >= COLS as i32) || !Self::can_place(&next, g, self.id) {
            self.write_to_grid(g);
            return false;
        }

        self.cells = next;
        self.write_to_grid(g);
        true
    }

    pub fn drop(&mut self, g: &mut Grid) {
        while self.move_down(g) {}
    }

    pub fn rotate_cw(&mut self, g: &mut Grid) {
        self.rotate(g, true);
    }

    pub fn rotate_ccw(&mut self, g: &mut Grid) {
        self.rotate(g, false);
    }

    fn rotate(&mut self, g: &mut Grid, cw: bool) {
        // Clear current block from grid so collision checks don't see itself.
        self.clear_from_grid(g);

        let (mut min_r, mut max_r) = (i32::MAX, i32::MIN);
        let (mut min_c, mut max_c) = (i32::MAX, i32::MIN);
        for e in &self.cells {
            min_r = min_r.min(e.r);
            max_r = max_r.max(e.r);
            min_c = min_c.min(e.c);
            max_c = max_c.max(e.c);
        }

        let h = (max_r - min_r + 1) as usize;
        let w = (max_c - min_c + 1) as usize;

        let mut local = vec![vec![' '; w]; h];
        for e in &self.cells {
            let lr = (e.r - min_r) as usize;
            let lc = (e.c - min_c) as usize;
            local[lr][lc] = e.ch;
        }

        let mut rot = vec![vec![' '; h]; w];
        if cw {
            for r in 0..h {
                for c in 0..w {
                    rot[c][h - 1 - r] = local[r][c];
                }
            }
        } else {
            for r in 0..h {
                for c in 0..w {
                    rot[w - 1 - c][r] = local[r][c];
                }
            }
        }

        // same anchoring logic as the C++: base at (max_r, min_c)
        let base_r = max_r;
        let base_c = min_c;

        let mut new_cells: Vec<Cell> = Vec::new();
        for r in 0..w {
            for c in 0..h {
                if rot[r][c] != ' ' {
                    let nr = base_r - ((w - 1 - r) as i32);
                    let nc = base_c + (c as i32);
                    new_cells.push(Cell { r: nr, c: nc, ch: rot[r][c] });
                }
            }
        }

        if new_cells.len() != self.cells.len() || !Self::can_place(&new_cells, g, self.id) {
            // Revert
            self.write_to_grid(g);
            return;
        }

        self.cells = new_cells;
        self.write_to_grid(g);
    }
}

fn spawn_cells(kind: BlockKind) -> Vec<Cell> {
    match kind {
        BlockKind::L => vec![
            Cell { r: 8, c: 0, ch: 'L' },
            Cell { r: 8, c: 1, ch: 'L' },
            Cell { r: 8, c: 2, ch: 'L' },
            Cell { r: 7, c: 2, ch: 'L' },
        ],
        BlockKind::I => vec![
            Cell { r: 7, c: 0, ch: 'I' },
            Cell { r: 7, c: 1, ch: 'I' },
            Cell { r: 7, c: 2, ch: 'I' },
            Cell { r: 7, c: 3, ch: 'I' },
        ],
        BlockKind::J => vec![
            Cell { r: 7, c: 0, ch: 'J' },
            Cell { r: 8, c: 0, ch: 'J' },
            Cell { r: 8, c: 1, ch: 'J' },
            Cell { r: 8, c: 2, ch: 'J' },
        ],
        BlockKind::Z => vec![
            Cell { r: 7, c: 0, ch: 'Z' },
            Cell { r: 7, c: 1, ch: 'Z' },
            Cell { r: 8, c: 1, ch: 'Z' },
            Cell { r: 8, c: 2, ch: 'Z' },
        ],
        BlockKind::S => vec![
            Cell { r: 8, c: 0, ch: 'S' },
            Cell { r: 8, c: 1, ch: 'S' },
            Cell { r: 7, c: 1, ch: 'S' },
            Cell { r: 7, c: 2, ch: 'S' },
        ],
        BlockKind::T => vec![
            Cell { r: 7, c: 0, ch: 'T' },
            Cell { r: 7, c: 1, ch: 'T' },
            Cell { r: 7, c: 2, ch: 'T' },
            Cell { r: 8, c: 1, ch: 'T' },
        ],
        BlockKind::O => vec![
            Cell { r: 7, c: 0, ch: 'O' },
            Cell { r: 7, c: 1, ch: 'O' },
            Cell { r: 8, c: 0, ch: 'O' },
            Cell { r: 8, c: 1, ch: 'O' },
        ],
        BlockKind::Star => vec![
            Cell { r: 7, c: 5, ch: '*' }
        ],
    }
}
