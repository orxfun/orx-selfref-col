use super::{refs::Refs, NodePtr};
use crate::variant::Variant;
use core::fmt::Debug;

/// A constant number of references.
pub struct RefsArray<const N: usize, V>([Option<NodePtr<V>>; N])
where
    V: Variant;

impl<const N: usize, V: Variant> Clone for RefsArray<N, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<const N: usize, V: Variant> Debug for RefsArray<N, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("RefsArray").field(&self.0).finish()
    }
}

impl<const N: usize, V> Refs for RefsArray<N, V>
where
    V: Variant,
{
    fn empty() -> Self {
        Self([const { None }; N])
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|x| x.is_none())
    }

    fn clear(&mut self) {
        self.0.iter_mut().for_each(|x| _ = x.take());
    }
}

impl<const N: usize, V: Variant> RefsArray<N, V> {
    /// Returns the node pointer a the `ref_idx` position of the references array.
    pub fn get(&self, ref_idx: usize) -> Option<NodePtr<V>> {
        self.0[ref_idx].clone()
    }

    // mut

    /// Sets the the node pointer a the `ref_idx` position of the references array to the given `node_idx`.
    pub fn set(&mut self, ref_idx: usize, node_idx: Option<NodePtr<V>>) {
        self.0[ref_idx] = node_idx;
    }

    /// Sets the the node pointer a the `ref_idx` position of the references array to the given `node_idx`.
    pub fn set_some(&mut self, ref_idx: usize, node_idx: &NodePtr<V>) {
        self.0[ref_idx] = Some(node_idx.clone())
    }

    /// Un-sets the the node pointer a the `ref_idx` position of the references array.
    pub fn set_none(&mut self, ref_idx: usize) {
        self.0[ref_idx] = None
    }
}
