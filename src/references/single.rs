use super::{refs::Refs, NodePtr};
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
    fn empty() -> Self {
        Self(None)
    }

    fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    fn clear(&mut self) {
        _ = self.0.take();
    }
}

impl<V: Variant> RefsSingle<V> {
    /// Returns the pointer to the referenced node.
    pub fn get(&self) -> Option<&NodePtr<V>> {
        self.0.as_ref()
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
