use super::{NodePtr, refs::Refs};
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
    #[inline(always)]
    fn empty() -> Self {
        Self(Vec::new())
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.0.clear();
    }

    #[inline(always)]
    fn remove_at(&mut self, ref_idx: usize) {
        self.0.remove(ref_idx);
    }

    #[inline(always)]
    fn remove(&mut self, ptr: usize) -> Option<usize> {
        let x = self
            .0
            .iter()
            .enumerate()
            .find(|x| unsafe { x.1.ptr() } as usize == ptr);
        match x {
            Some((ref_idx, _)) => {
                self.0.remove(ref_idx);
                Some(ref_idx)
            }
            None => None,
        }
    }
}

impl<V: Variant> RefsVec<V> {
    /// Returns the number of references.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the node pointers as a slice.
    pub fn as_slice(&self) -> &[NodePtr<V>] {
        self.0.as_slice()
    }

    /// Returns true if the number of references is zero.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the i-th node pointer; None if out of bounds.
    pub fn get(&self, i: usize) -> Option<&NodePtr<V>> {
        self.0.get(i)
    }

    /// Creates an iterator over node pointers of the references collection.
    pub fn iter(&self) -> core::slice::Iter<'_, NodePtr<V>> {
        self.0.iter()
    }

    /// Creates a mutable iterator over node pointers of the references collection.
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, NodePtr<V>> {
        self.0.iter_mut()
    }

    /// Pushes the node references to the end of the references collection.
    pub fn push(&mut self, node_ptr: NodePtr<V>) {
        self.0.push(node_ptr);
    }

    /// Inserts the reference with the given `node_ptr` to the given `position` of the references collection.
    pub fn insert(&mut self, position: usize, node_ptr: NodePtr<V>) {
        self.0.insert(position, node_ptr);
    }

    /// Inserts the reference with the given `node_ptr` just before the given `pivot_ptr` the reference if it exists;
    /// and returns the position that the new reference is inserted to.
    ///
    /// Does nothing leaving the children unchanged if the `pivot_ptr` reference does not exists, and returns None.
    pub fn push_before(&mut self, pivot_ptr: &NodePtr<V>, node_ptr: NodePtr<V>) -> Option<usize> {
        let position = self.iter().position(|x| x == pivot_ptr);
        if let Some(p) = position {
            self.0.insert(p, node_ptr);
        }
        position
    }

    /// Inserts the reference with the given `node_ptr` just after the given `pivot_ptr` the reference if it exists;
    /// and returns the position that the new reference is inserted to.
    ///
    /// Does nothing leaving the children unchanged if the `pivot_ptr` reference does not exists, and returns None.
    pub fn push_after(&mut self, pivot_ptr: &NodePtr<V>, node_ptr: NodePtr<V>) -> Option<usize> {
        let position = self.iter().position(|x| x == pivot_ptr);
        if let Some(p) = position {
            self.0.insert(p + 1, node_ptr);
        }
        position
    }

    /// Replaces the node reference `old_node_ptr` with the `new_node_ptr` and returns
    /// the position of the reference.
    ///
    /// Does nothing and returns None if the `old_node_ptr` is absent.
    pub fn replace_with(
        &mut self,
        old_node_ptr: &NodePtr<V>,
        new_node_ptr: NodePtr<V>,
    ) -> Option<usize> {
        let position = self.0.iter().position(|x| x == old_node_ptr);
        if let Some(p) = position {
            self.0[p] = new_node_ptr;
        }
        position
    }
}
