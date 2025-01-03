use core::fmt::Debug;

/// References among nodes.
pub trait Refs: Clone + Debug {
    /// Creates an empty references.
    fn empty() -> Self;

    /// Returns true if the references collection is empty.
    fn is_empty(&self) -> bool;

    /// Clears the references.
    fn clear(&mut self);

    /// Removes the reference at the given `ref_idx`.
    fn remove_at(&mut self, ref_idx: usize);

    /// Removes the node reference from references pointing to the node at given `ptr` location.
    ///
    /// Returns the position of the `ptr` among references if it exists; None otherwise.
    fn remove(&mut self, ptr: usize) -> Option<usize>;
}
