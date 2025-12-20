#[derive(Debug, Clone)]
pub enum Effect {
    Blind { expired: bool },
    Heavy { expired: bool },
}

impl Effect {
    pub fn blind() -> Self {
        Effect::Blind { expired: false }
    }
    pub fn heavy() -> Self {
        Effect::Heavy { expired: false }
    }

    pub fn is_blind(&self) -> bool {
        matches!(self, Effect::Blind { .. })
    }

    pub fn adds_heavy_on_horizontal(&self) -> bool {
        matches!(self, Effect::Heavy { .. })
    }

    pub fn on_drop(&mut self) {
        match self {
            Effect::Blind { expired } => *expired = true,
            Effect::Heavy { expired } => *expired = true,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self {
            Effect::Blind { expired } => *expired,
            Effect::Heavy { expired } => *expired,
        }
    }
}
