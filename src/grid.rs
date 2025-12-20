use std::collections::HashMap;

pub const COLS: usize = 11;
pub const ROWS_TOTAL: usize = 26;
pub const PLAY_TOP: usize = 4;
pub const PLAY_BOTTOM: usize = 21;

#[derive(Clone)]
pub struct Grid {
    cur_score: i32,
    matrix: Vec<Vec<char>>,
    block_id: Vec<Vec<i32>>,
}

impl Grid {
    pub fn new() -> Self {
        let mut matrix = vec![vec![' '; COLS]; ROWS_TOTAL];

        // Row 0 is unused in the C++ print routine, but we keep it.
        put_row(&mut matrix, 0, "Hi Score: 0");
        put_row(&mut matrix, 1, "Level:    0");
        put_row(&mut matrix, 2, "Score:    0");
        put_row(&mut matrix, 3, "-----------");

        for r in PLAY_TOP..=PLAY_BOTTOM {
            for c in 0..COLS {
                matrix[r][c] = ' ';
            }
        }

        put_row(&mut matrix, 22, "-----------");
        put_row(&mut matrix, 23, "Next:      ");
        put_row(&mut matrix, 24, "           ");
        put_row(&mut matrix, 25, "           ");

        let block_id = vec![vec![-1; COLS]; ROWS_TOTAL];

        Grid { cur_score: 0, matrix, block_id }
    }

    pub fn rows_total(&self) -> usize { self.matrix.len() }

    pub fn get(&self, r: usize, c: usize) -> char {
        self.matrix[r][c]
    }

    pub fn get_block_id(&self, r: usize, c: usize) -> i32 {
        if r >= self.block_id.len() || c >= self.block_id[0].len() {
            return -1;
        }
        self.block_id[r][c]
    }

    pub fn set_matrix(&mut self, r: usize, c: usize, ch: char) {
        self.matrix[r][c] = ch;
        if ch == ' ' && (PLAY_TOP..=PLAY_BOTTOM).contains(&r) {
            self.block_id[r][c] = -1;
        }
    }

    pub fn set_cell(&mut self, r: usize, c: usize, ch: char, bid: i32) {
        self.matrix[r][c] = ch;
        if (PLAY_TOP..=PLAY_BOTTOM).contains(&r) {
            self.block_id[r][c] = bid;
        }
    }

    pub fn clear_cell(&mut self, r: usize, c: usize) {
        self.matrix[r][c] = ' ';
        if (PLAY_TOP..=PLAY_BOTTOM).contains(&r) {
            self.block_id[r][c] = -1;
        }
    }

    pub fn matrix(&self) -> &Vec<Vec<char>> {
        &self.matrix
    }

    pub fn cur_score(&self) -> i32 {
        self.cur_score
    }

    // Accumulates (matches your C++ behavior)
    pub fn add_score(&mut self, delta: i32) {
        self.cur_score += delta;

        // clear score display columns 8..=10
        for i in 8..=10 {
            self.matrix[2][i] = ' ';
        }
        let s = self.cur_score.to_string();
        let mut pos = 11usize.saturating_sub(s.len());
        for ch in s.chars() {
            if pos < 11 {
                self.matrix[2][pos] = ch;
                pos += 1;
            }
        }
    }

    pub fn set_level_digit(&mut self, lvl: i32) {
        let d = char::from(b'0' + (lvl as u8));
        self.matrix[1][10] = d;
    }

    pub fn show_next(&mut self, preview_block_cells: &[(i32, i32, char)]) {
        // Clear rows 24..=25
        for r in 24..=25 {
            for c in 0..COLS {
                self.matrix[r][c] = ' ';
            }
        }

        let base_r = 17; // maps 7->24, 8->25
        for (r, c, ch) in preview_block_cells {
            let rr = (*r + base_r) as i32;
            let cc = *c;
            if rr >= 0 && (rr as usize) < self.matrix.len() && cc >= 0 && (cc as usize) < COLS {
                self.matrix[rr as usize][cc as usize] = *ch;
            }
        }
    }

    pub fn check_and_clear(&mut self, block_loss: &mut HashMap<i32, i32>) -> i32 {
        let top = PLAY_TOP;
        let bottom = PLAY_BOTTOM;

        let mut rows_cleared: i32 = 0;
        let mut write_row: i32 = bottom as i32;

        for row in (top..=bottom).rev() {
            let mut full = true;
            for col in 0..COLS {
                if self.matrix[row][col] == ' ' {
                    full = false;
                    break;
                }
            }

            if !full {
                if write_row as usize != row {
                    for col in 0..COLS {
                        self.matrix[write_row as usize][col] = self.matrix[row][col];
                        self.block_id[write_row as usize][col] = self.block_id[row][col];
                    }
                }
                write_row -= 1;
            } else {
                rows_cleared += 1;
                for col in 0..COLS {
                    let bid = self.block_id[row][col];
                    if bid > 0 {
                        *block_loss.entry(bid).or_insert(0) += 1;
                    }
                }
            }
        }

        for row in (top as i32..=write_row).rev() {
            for col in 0..COLS {
                self.matrix[row as usize][col] = ' ';
                self.block_id[row as usize][col] = -1;
            }
        }

        rows_cleared
    }
}

fn put_row(matrix: &mut [Vec<char>], r: usize, s: &str) {
    let chars: Vec<char> = s.chars().collect();
    for c in 0..COLS {
        matrix[r][c] = if c < chars.len() { chars[c] } else { ' ' };
    }
}
