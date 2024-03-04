#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct MemoryState {
    pub id: usize,
}

impl MemoryState {
    pub const fn new() -> Self {
        Self { id: 0 }
    }

    pub fn successor_state(&self) -> Self {
        Self { id: self.id + 1 }
    }
}

impl Default for MemoryState {
    fn default() -> Self {
        Self::new()
    }
}
