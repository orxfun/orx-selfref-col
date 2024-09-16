/// Memory state of a self referential collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MemoryState {
    pub(crate) id: usize,
}

impl MemoryState {
    pub(crate) const fn successor_state(&self) -> Self {
        Self { id: self.id + 1 }
    }
}
