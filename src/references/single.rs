use super::{NodePtr, refs::Refs};
use crate::variant::Variant;
use core::fmt::Debug;

/// A single node reference.
pub struct RefsSingle<V>(Option<NodePtr<V>>)
where
    V: Variant;

impl<V: Variant> Clone for RefsSingle<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V: Variant> Debug for RefsSingle<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("RefsSingle").field(&self.0).finish()
    }
}

impl<V: Variant> Refs for RefsSingle<V> {
    #[inline(always)]
    fn empty() -> Self {
        Self(None)
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    #[inline(always)]
    fn clear(&mut self) {
        _ = self.0.take();
    }

    #[inline(always)]
    fn remove_at(&mut self, ref_idx: usize) {
        assert_eq!(
            ref_idx, 0,
            "Reference idx {ref_idx} is out of bounds for RefsSingle.",
        );
        self.clear();
    }

    #[inline(always)]
    fn remove(&mut self, ptr: usize) -> Option<usize> {
        match &mut self.0 {
            None => None,
            Some(x) => match unsafe { x.ptr() } as usize == ptr {
                true => {
                    self.clear();
                    Some(0)
                }
                false => None,
            },
        }
    }
}

impl<V: Variant> RefsSingle<V> {
    /// Returns the pointer to the referenced node.
    pub fn get(&self) -> Option<NodePtr<V>> {
        self.0
    }

    /// Sets the pointer to the referenced node with the given `node_idx`.
    pub fn set(&mut self, node_idx: Option<NodePtr<V>>) {
        self.0 = node_idx
    }

    /// Sets the pointer to the referenced node with the given `node_idx`.
    pub fn set_some(&mut self, node_idx: NodePtr<V>) {
        self.0 = Some(node_idx)
    }

    /// Un-sets the reference.
    pub fn set_none(&mut self) {
        self.0 = None
    }
}
