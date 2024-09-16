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
