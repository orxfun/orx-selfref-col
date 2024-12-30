use super::{
    iter::{ArrayLeftMostPtrIter, ArrayLeftMostPtrIterMut},
    refs::Refs,
    NodePtr,
};
use crate::variant::Variant;
use core::fmt::Debug;

/// A constant number of left-aligned references.
///
/// It differs from [`RefsArray`] due to its additional requirement that:
/// * all Some references are to the left of all None references.
///
/// [`RefsArray`]: crate::RefsArray
pub struct RefsArrayLeftMost<const N: usize, V>
where
    V: Variant,
{
    array: [Option<NodePtr<V>>; N],
    len: usize,
}

impl<const N: usize, V: Variant> Clone for RefsArrayLeftMost<N, V> {
    fn clone(&self) -> Self {
        Self {
            array: self.array.clone(),
            len: self.len,
        }
    }
}

impl<const N: usize, V: Variant> Debug for RefsArrayLeftMost<N, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("RefsArrayLeftMost")
            .field(&self.array)
            .finish()
    }
}

impl<const N: usize, V> Refs for RefsArrayLeftMost<N, V>
where
    V: Variant,
{
    fn empty() -> Self {
        Self {
            array: [const { None }; N],
            len: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn clear(&mut self) {
        self.array
            .iter_mut()
            .take(self.len)
            .for_each(|x| _ = x.take());
        self.len = 0;
    }
}

impl<const N: usize, V: Variant> RefsArrayLeftMost<N, V> {
    /// Returns the number of references.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a reference to the node pointer at the `ref_idx` position of the references array.
    pub fn get(&self, ref_idx: usize) -> Option<&NodePtr<V>> {
        self.array[ref_idx].as_ref()
    }

    /// Returns a mutable reference to the node pointer at the `ref_idx` position of the references array.
    pub fn get_mut(&mut self, ref_idx: usize) -> Option<&mut NodePtr<V>> {
        self.array[ref_idx].as_mut()
    }

    /// Creates an iterator over node pointers of the references collection.
    pub fn iter(&self) -> ArrayLeftMostPtrIter<V> {
        let slice = &self.array[..self.len];
        ArrayLeftMostPtrIter::new(slice.iter())
    }

    /// Creates a mutable iterator over node pointers of the references collection.
    pub fn iter_mut(&mut self) -> ArrayLeftMostPtrIterMut<V> {
        let slice = &mut self.array[..self.len];
        ArrayLeftMostPtrIterMut::new(slice.iter_mut())
    }

    /// Returns whether or not the collection has room for another reference.
    pub fn has_room(&self) -> bool {
        self.len < N
    }

    /// Pushes the node references to the end of the references collection.
    ///
    /// # Panics
    ///
    /// Panics if the array already has `N` references; i.e., when `self.has_room()` is false.
    pub fn push(&mut self, node_ptr: NodePtr<V>) {
        assert!(
            self.has_room(),
            "Pushing the {}-th reference to array-backed references with maximum of {} elements.",
            N + 1,
            N
        );
        self.array[self.len] = Some(node_ptr);
        self.len += 1;
    }
}
