use core::fmt::Debug;

/// References among nodes.
pub trait Refs: Clone + Debug {
    /// Creates an empty references.
    fn empty() -> Self;

    /// Returns true if the references collection is empty.
    fn is_empty(&self) -> bool;

    /// Clears the references.
    fn clear(&mut self);
}
