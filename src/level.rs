use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;

use crate::block::BlockKind;

#[derive(Clone)]
pub enum Level {
    Zero(Level0),
    Random(RandomLevel),
    Four(Level4),
}

impl Level {
    pub fn number(&self) -> i32 {
        match self {
            Level::Zero(_) => 0,
            Level::Random(r) => r.level,
            Level::Four(f) => f.level,
        }
    }

    pub fn is_heavy(&self) -> bool {
        match self {
            Level::Zero(_) => false,
            Level::Random(r) => r.heavy,
            Level::Four(f) => f.heavy,
        }
    }

    /// Peek: does NOT advance sequence pointer (except random)
    pub fn peek_kind(&mut self, rng: &mut StdRng) -> BlockKind {
        match self {
            Level::Zero(l0) => l0.peek(),
            Level::Random(rl) => rl.peek(rng),
            Level::Four(l4) => l4.peek(rng),
        }
    }

    /// Advance: advances sequence pointer (or samples again for random)
    pub fn advance_kind(&mut self, rng: &mut StdRng) -> BlockKind {
        match self {
            Level::Zero(l0) => l0.advance(),
            Level::Random(rl) => rl.advance(rng),
            Level::Four(l4) => l4.advance(rng),
        }
    }

    pub fn notify_rows_cleared(&mut self, cleared: i32) {
        if let Level::Four(l4) = self {
            l4.notify_rows_cleared(cleared);
        }
    }

    pub fn notify_block_placed(&mut self) {
        if let Level::Four(l4) = self {
            l4.notify_block_placed();
        }
    }

    pub fn should_drop_star(&self) -> bool {
        match self {
            Level::Four(l4) => l4.should_drop_star(),
            _ => false,
        }
    }

    pub fn set_random(&mut self, val: bool) -> Result<(), String> {
        match self {
            Level::Random(rl) if rl.level >= 3 => { rl.use_random = val; Ok(()) }
            Level::Four(l4) => { l4.use_random = val; Ok(()) }
            Level::Random(_) => Err("random/norandom is only relevant in levels 3 and 4".to_string()),
            Level::Zero(_) => Err("random/norandom is only relevant in levels 3 and 4".to_string()),
        }
    }

    pub fn load_sequence(&mut self, file: &str) -> Result<(), String> {
        match self {
            Level::Random(rl) if rl.level >= 3 => rl.load_sequence(file),
            Level::Four(l4) => l4.load_sequence(file),
            _ => Err("norandom is only relevant in levels 3 and 4".to_string()),
        }
    }
}

#[derive(Clone)]
pub struct Level0 {
    order: Vec<BlockKind>,
    pos: usize,
}

impl Level0 {
    pub fn from_file(file: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(file)
            .map_err(|_| format!("Cannot open sequence file: {}", file))?;
        let mut order = Vec::new();
        for tok in content.split_whitespace() {
            if tok.len() != 1 { continue; }
            let ch = tok.chars().next().unwrap();
            if let Some(k) = BlockKind::from_char(ch) {
                if k != BlockKind::Star {
                    order.push(k);
                }
            }
        }
        if order.is_empty() {
            return Err(format!("sequence file empty: {}", file));
        }
        Ok(Level0 { order, pos: 0 })
    }

    pub fn peek(&self) -> BlockKind {
        self.order[self.pos]
    }

    pub fn advance(&mut self) -> BlockKind {
        self.pos = (self.pos + 1) % self.order.len();
        self.order[self.pos]
    }
}

#[derive(Clone)]
pub struct RandomLevel {
    pub level: i32,
    pub heavy: bool,
    weights: Vec<usize>,
    pub use_random: bool,
    seq: Vec<BlockKind>,
    seq_pos: usize,
}

impl RandomLevel {
    pub fn new(level: i32, heavy: bool, weights: Vec<usize>) -> Self {
        Self {
            level,
            heavy,
            weights,
            use_random: true,
            seq: Vec::new(),
            seq_pos: 0,
        }
    }

    fn sample(&self, rng: &mut StdRng) -> BlockKind {
        // Index mapping matches the C++ builder order: 0 T, 1 S, 2 Z, 3 I, 4 J, 5 L, 6 O
        let dist = WeightedIndex::new(self.weights.clone()).unwrap();
        match dist.sample(rng) {
            0 => BlockKind::T,
            1 => BlockKind::S,
            2 => BlockKind::Z,
            3 => BlockKind::I,
            4 => BlockKind::J,
            5 => BlockKind::L,
            6 => BlockKind::O,
            _ => BlockKind::T,
        }
    }

    pub fn load_sequence(&mut self, file: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(file)
            .map_err(|_| format!("Cannot open sequence file: {}", file))?;
        let mut seq = Vec::new();
        for tok in content.split_whitespace() {
            if tok.len() != 1 { continue; }
            let ch = tok.chars().next().unwrap();
            if let Some(k) = BlockKind::from_char(ch) {
                if k != BlockKind::Star {
                    seq.push(k);
                }
            }
        }
        if seq.is_empty() {
            return Err(format!("sequence file empty: {}", file));
        }
        self.seq = seq;
        self.seq_pos = 0;
        Ok(())
    }

    pub fn peek(&mut self, rng: &mut StdRng) -> BlockKind {
        if !self.use_random && !self.seq.is_empty() {
            self.seq[self.seq_pos]
        } else {
            self.sample(rng)
        }
    }

    pub fn advance(&mut self, rng: &mut StdRng) -> BlockKind {
        if !self.use_random && !self.seq.is_empty() {
            self.seq_pos = (self.seq_pos + 1) % self.seq.len();
            self.seq[self.seq_pos]
        } else {
            self.sample(rng)
        }
    }
}

#[derive(Clone)]
pub struct Level4 {
    pub level: i32,
    pub heavy: bool,
    weights: Vec<usize>,
    pub use_random: bool,
    seq: Vec<BlockKind>,
    seq_pos: usize,
    blocks_since_clear: i32,
}

impl Level4 {
    pub fn new() -> Self {
        Self {
            level: 4,
            heavy: true,
            // same weights as C++ LevelFour
            weights: vec![1, 2, 2, 1, 1, 1, 1],
            use_random: true,
            seq: Vec::new(),
            seq_pos: 0,
            blocks_since_clear: 0,
        }
    }

    fn sample(&self, rng: &mut StdRng) -> BlockKind {
        let dist = WeightedIndex::new(self.weights.clone()).unwrap();
        match dist.sample(rng) {
            0 => BlockKind::T,
            1 => BlockKind::S,
            2 => BlockKind::Z,
            3 => BlockKind::I,
            4 => BlockKind::J,
            5 => BlockKind::L,
            6 => BlockKind::O,
            _ => BlockKind::T,
        }
    }

    pub fn load_sequence(&mut self, file: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(file)
            .map_err(|_| format!("Cannot open sequence file: {}", file))?;
        let mut seq = Vec::new();
        for tok in content.split_whitespace() {
            if tok.len() != 1 { continue; }
            let ch = tok.chars().next().unwrap();
            if let Some(k) = BlockKind::from_char(ch) {
                if k != BlockKind::Star {
                    seq.push(k);
                }
            }
        }
        if seq.is_empty() {
            return Err(format!("sequence file empty: {}", file));
        }
        self.seq = seq;
        self.seq_pos = 0;
        Ok(())
    }

    pub fn peek(&mut self, rng: &mut StdRng) -> BlockKind {
        if !self.use_random && !self.seq.is_empty() {
            self.seq[self.seq_pos]
        } else {
            self.sample(rng)
        }
    }

    pub fn advance(&mut self, rng: &mut StdRng) -> BlockKind {
        if !self.use_random && !self.seq.is_empty() {
            self.seq_pos = (self.seq_pos + 1) % self.seq.len();
            self.seq[self.seq_pos]
        } else {
            self.sample(rng)
        }
    }

    pub fn notify_rows_cleared(&mut self, cleared: i32) {
        if cleared > 0 {
            self.blocks_since_clear = 0;
        }
    }

    pub fn notify_block_placed(&mut self) {
        self.blocks_since_clear += 1;
    }

    pub fn should_drop_star(&self) -> bool {
        self.blocks_since_clear > 0 && (self.blocks_since_clear % 5 == 0)
    }
}

pub fn generate_level(level: i32, script_file: &str) -> Result<Level, String> {
    match level {
        1 => Ok(Level::Random(RandomLevel::new(1, false, vec![2, 1, 1, 2, 2, 2, 2]))),
        2 => Ok(Level::Random(RandomLevel::new(2, false, vec![1, 1, 1, 1, 1, 1, 1]))),
        3 => Ok(Level::Random(RandomLevel::new(3, true,  vec![1, 2, 2, 1, 1, 1, 1]))),
        4 => Ok(Level::Four(Level4::new())),
        _ => Ok(Level::Zero(Level0::from_file(script_file)?)),
    }
}
