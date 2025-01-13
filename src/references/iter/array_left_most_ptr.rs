use crate::{NodePtr, Variant};

/// Iterator for active references of an [`RefsArrayLeftMost`] collection,
/// which can be created by its `iter` method.
///
/// [`RefsArrayLeftMost`]: crate::RefsArrayLeftMost
pub struct ArrayLeftMostPtrIter<'a, V: Variant> {
    iter: core::slice::Iter<'a, Option<NodePtr<V>>>,
}

impl<'a, V: Variant> ArrayLeftMostPtrIter<'a, V> {
    pub(crate) fn new(iter: core::slice::Iter<'a, Option<NodePtr<V>>>) -> Self {
        Self { iter }
    }
}

impl<V: Variant> Default for ArrayLeftMostPtrIter<'_, V> {
    fn default() -> Self {
        Self {
            iter: Default::default(),
        }
    }
}

impl<'a, V: Variant> Iterator for ArrayLeftMostPtrIter<'a, V> {
    type Item = &'a NodePtr<V>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|x| x.as_ref())
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.len();
        (len, Some(len))
    }
}

impl<V: Variant> ExactSizeIterator for ArrayLeftMostPtrIter<'_, V> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<V: Variant> DoubleEndedIterator for ArrayLeftMostPtrIter<'_, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().and_then(|x| x.as_ref())
    }
}

// mut

/// Mutable iterator for active references of an [`RefsArrayLeftMost`] collection,
/// which can be created by its `iter_mut` method.
///
/// [`RefsArrayLeftMost`]: crate::RefsArrayLeftMost
pub struct ArrayLeftMostPtrIterMut<'a, V: Variant> {
    iter: core::slice::IterMut<'a, Option<NodePtr<V>>>,
}

impl<'a, V: Variant> ArrayLeftMostPtrIterMut<'a, V> {
    pub(crate) fn new(iter: core::slice::IterMut<'a, Option<NodePtr<V>>>) -> Self {
        Self { iter }
    }
}

impl<V: Variant> Default for ArrayLeftMostPtrIterMut<'_, V> {
    fn default() -> Self {
        Self {
            iter: Default::default(),
        }
    }
}

impl<'a, V: Variant> Iterator for ArrayLeftMostPtrIterMut<'a, V> {
    type Item = &'a mut NodePtr<V>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|x| x.as_mut())
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.len();
        (len, Some(len))
    }
}

impl<V: Variant> ExactSizeIterator for ArrayLeftMostPtrIterMut<'_, V> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<V: Variant> DoubleEndedIterator for ArrayLeftMostPtrIterMut<'_, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().and_then(|x| x.as_mut())
    }
}
