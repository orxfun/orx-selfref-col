use super::{
    NodePtr,
    iter::{ArrayLeftMostPtrIter, ArrayLeftMostPtrIterMut},
    refs::Refs,
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
    #[inline(always)]
    fn empty() -> Self {
        Self {
            array: [const { None }; N],
            len: 0,
        }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.array
            .iter_mut()
            .take(self.len)
            .for_each(|x| _ = x.take());
        self.len = 0;
    }

    #[inline(always)]
    fn remove_at(&mut self, ref_idx: usize) {
        self.array[ref_idx] = None;
        for i in (ref_idx + 1)..self.len {
            self.array[i - 1] = self.array[i].take();
        }
        self.len -= 1;
    }

    #[inline(always)]
    fn remove(&mut self, ptr: usize) -> Option<usize> {
        let x = self.array.iter().enumerate().find(|x| match x.1 {
            Some(x) => x.ptr() as usize == ptr,
            None => false,
        });
        match x {
            Some((ref_idx, _)) => {
                self.remove_at(ref_idx);
                Some(ref_idx)
            }
            None => None,
        }
    }
}

impl<const N: usize, V: Variant> RefsArrayLeftMost<N, V> {
    /// Returns the number of references.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the number of references is zero.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the node pointer at the `ref_idx` position of the references array.
    pub fn get(&self, ref_idx: usize) -> Option<&NodePtr<V>> {
        match ref_idx < N {
            true => self.array[ref_idx].as_ref(),
            false => None,
        }
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
        self.assert_has_room_for::<1>();
        self.array[self.len] = Some(node_ptr);
        self.len += 1;
    }

    /// Inserts the reference with the given `node_ptr` to the given `position` of the references collection.
    pub fn insert(&mut self, position: usize, node_ptr: NodePtr<V>) {
        for q in (position..self.len).rev() {
            self.array[q + 1] = self.array[q].clone();
        }
        self.array[position] = Some(node_ptr);
        self.len += 1;
    }

    /// Inserts the reference with the given `node_ptr` just before the given `pivot_ptr` the reference if it exists;
    /// and returns the position that the new reference is inserted to.
    ///
    /// Does nothing leaving the children unchanged if the `pivot_ptr` reference does not exists, and returns None.
    pub fn push_before(&mut self, pivot_ptr: &NodePtr<V>, node_ptr: NodePtr<V>) -> Option<usize> {
        self.assert_has_room_for::<1>();
        let position = self.iter().position(|x| x == pivot_ptr);
        if let Some(p) = position {
            self.insert(p, node_ptr);
        }
        position
    }

    /// Inserts the reference with the given `node_ptr` just after the given `pivot_ptr` the reference if it exists;
    /// and returns the position that the new reference is inserted to.
    ///
    /// Does nothing leaving the children unchanged if the `pivot_ptr` reference does not exists, and returns None.
    pub fn push_after(&mut self, pivot_ptr: &NodePtr<V>, node_ptr: NodePtr<V>) -> Option<usize> {
        self.assert_has_room_for::<1>();
        let position = self.iter().position(|x| x == pivot_ptr);
        if let Some(p) = position {
            self.insert(p + 1, node_ptr);
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
        let position = self.iter().position(|x| x == old_node_ptr);
        if let Some(p) = position {
            self.array[p] = Some(new_node_ptr);
        }
        position
    }

    // helpers
    fn assert_has_room_for<const P: usize>(&self) {
        assert!(
            self.len + P <= N,
            "Pushing the {}-th reference to array-backed references with maximum of {} elements.",
            N + P,
            N
        );
    }
}
