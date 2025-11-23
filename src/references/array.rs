use super::{NodePtr, refs::Refs};
use crate::variant::Variant;
use core::fmt::Debug;

/// A constant number of references.
pub struct RefsArray<const N: usize, V>([Option<NodePtr<V>>; N])
where
    V: Variant;

impl<const N: usize, V: Variant> Clone for RefsArray<N, V> {
    fn clone(&self) -> Self {
        Self(self.0)
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
    #[inline(always)]
    fn empty() -> Self {
        Self([const { None }; N])
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0.iter().all(|x| x.is_none())
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.0.iter_mut().for_each(|x| _ = x.take());
    }

    #[inline(always)]
    fn remove_at(&mut self, ref_idx: usize) {
        self.0[ref_idx] = None;
    }

    #[inline(always)]
    fn remove(&mut self, ptr: usize) -> Option<usize> {
        let x = self.0.iter().enumerate().find(|x| match x.1 {
            Some(x) => unsafe { x.ptr() as usize == ptr },
            None => false,
        });
        match x {
            Some((ref_idx, _)) => {
                self.0[ref_idx] = None;
                Some(ref_idx)
            }
            None => None,
        }
    }
}

impl<const N: usize, V: Variant> RefsArray<N, V> {
    /// Returns the node pointers as a slice.
    pub fn as_slice(&self) -> &[Option<NodePtr<V>>] {
        self.0.as_slice()
    }

    /// Returns the node pointer at the `ref_idx` position of the references array.
    pub fn get(&self, ref_idx: usize) -> Option<NodePtr<V>> {
        self.0[ref_idx]
    }

    // mut

    /// Sets the the node pointer at the `ref_idx` position of the references array to the given `node_idx`.
    pub fn set(&mut self, ref_idx: usize, node_idx: Option<NodePtr<V>>) {
        self.0[ref_idx] = node_idx;
    }

    /// Sets the the node pointer at the `ref_idx` position of the references array to the given `node_idx`.
    pub fn set_some(&mut self, ref_idx: usize, node_idx: NodePtr<V>) {
        self.0[ref_idx] = Some(node_idx)
    }

    /// Un-sets the the node pointer at the `ref_idx` position of the references array.
    pub fn set_none(&mut self, ref_idx: usize) {
        self.0[ref_idx] = None
    }
}
