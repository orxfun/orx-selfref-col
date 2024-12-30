use super::{refs::Refs, NodePtr};
use crate::Variant;
use alloc::vec::Vec;
use core::fmt::Debug;

/// A dynamic number of references.
pub struct RefsVec<V>(Vec<NodePtr<V>>)
where
    V: Variant;

impl<V: Variant> Clone for RefsVec<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V: Variant> Debug for RefsVec<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("RefsVec").field(&self.0).finish()
    }
}

impl<V: Variant> Refs for RefsVec<V> {
    fn empty() -> Self {
        Self(Vec::new())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl<V: Variant> RefsVec<V> {
    /// Returns the number of references.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the i-th node pointer; None if out of bounds.
    pub fn get(&self, i: usize) -> Option<&NodePtr<V>> {
        self.0.get(i)
    }

    /// Creates an iterator over node pointers of the references collection.
    pub fn iter(&self) -> core::slice::Iter<NodePtr<V>> {
        self.0.iter()
    }

    /// Creates a mutable iterator over node pointers of the references collection.
    pub fn iter_mut(&mut self) -> core::slice::IterMut<NodePtr<V>> {
        self.0.iter_mut()
    }

    /// Pushes the node references to the end of the references collection.
    pub fn push(&mut self, node_ptr: NodePtr<V>) {
        self.0.push(node_ptr);
    }
}
