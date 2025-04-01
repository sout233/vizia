#[derive(Debug, Clone, PartialEq)]
pub enum ImeState {
    Inactive,
    StartComposition,
    Composing { preedit: Option<String>, cursor_pos: Option<(usize, usize)> },
    EndComposition,
}

impl Default for ImeState {
    fn default() -> Self {
        ImeState::Inactive
    }
}

impl ImeState {
    pub fn is_inactive(&self) -> bool {
        matches!(self, ImeState::Inactive)
    }

    pub fn is_composing(&self) -> bool {
        matches!(self, ImeState::Composing { .. })
    }
}
