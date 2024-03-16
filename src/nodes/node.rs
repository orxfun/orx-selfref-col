use crate::data::node_data::NodeData;
use crate::variants::memory_reclaim::Reclaim;
use crate::variants::variant::Variant;
use crate::{NodeDataLazyClose, NodeIndex, NodeRefsArray, NodeRefsVec, SelfRefColMut};
use orx_split_vec::prelude::PinnedVec;

use super::has_collection_key::HasCollectionKey;

/// A node of the self referential collection.
///
/// Each node is composed of the following three pieces of data:
/// * `data: V::Storage`: data of the element
/// * `prev: V::Prev`: references to the previous nodes in the collection
/// * `next: V::Next`: references to the previous nodes in the collection
pub struct Node<'a, V, T>
where
    V: Variant<'a, T>,
{
    pub(crate) data: V::Storage,
    pub(crate) prev: V::Prev,
    pub(crate) next: V::Next,
}

impl<'a, V, T> Node<'a, V, T>
where
    V: Variant<'a, T>,
{
    // new
    /// Creates a new free node with the given `data` without any previous or next references.
    #[inline(always)]
    pub fn new_free_node(data: T) -> Self {
        Self {
            data: V::Storage::active(data),
            prev: V::Prev::default(),
            next: V::Next::default(),
        }
    }

    /// Creates a new node with the given `data` and `prev` and `next` references.
    #[inline(always)]
    pub fn new(data: V::Storage, prev: V::Prev, next: V::Next) -> Self {
        Self { data, prev, next }
    }

    // get
    /// Returns whether the node is active, or closed otherwise.
    pub fn is_active(&self) -> bool {
        self.data.is_active()
    }

    /// Returns a reference to the data stored in the node; None if the node is closed.
    #[inline(always)]
    pub fn data(&self) -> Option<&T> {
        self.data.get()
    }

    /// Returns a reference to previous references of the node.
    #[inline(always)]
    pub fn prev(&self) -> &V::Prev {
        &self.prev
    }

    /// Returns a reference to next references of the node.
    #[inline(always)]
    pub fn next(&self) -> &V::Next {
        &self.next
    }

    // mut
    /// Returns a mutable reference to the data stored in the node; None if the node is closed.
    pub fn data_mut(&mut self) -> Option<&mut T> {
        self.data.get_mut()
    }

    // visit with key
    /// Gets the index of the node.
    ///
    /// This index can be:
    /// * stored independent of the self referential collection as a value,
    /// * used to safely access to this node in ***O(1)*** time.
    pub fn index<Collection>(&'a self, collection: &Collection) -> NodeIndex<'a, V, T>
    where
        Collection: HasCollectionKey<'a, V, T>,
    {
        NodeIndex::new(collection.collection_key(), self)
    }

    // mut with mut key
    /// Swaps the data stored in the node with the `new_value` and returns the old data.
    ///
    /// # Panics
    ///
    /// Panics if the node is closed.
    #[inline(always)]
    pub fn swap_data<'rf, P>(&'a self, vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>, new_value: T) -> T
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.swap_data(self, new_value)
    }

    /// Updates the previous references of the node as the given `prev`.
    #[inline(always)]
    pub fn set_prev_refs<'rf, P>(&'a self, vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>, prev: V::Prev)
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.set_prev_of(self, prev)
    }

    /// Updates the next references of the node as the given `next`.
    #[inline(always)]
    pub fn set_next_refs<'rf, P>(&'a self, vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>, next: V::Next)
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.set_next_of(self, next)
    }

    /// Updates the next reference of the node as the given `next`.
    #[inline(always)]
    pub fn set_next<'rf, P, Next: Into<V::Next>>(
        &'a self,
        vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>,
        next: Next,
    ) where
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.set_next_of(self, next.into())
    }

    /// Updates the previous reference of the node as the given `prev`.
    #[inline(always)]
    pub fn set_prev<'rf, P, Prev: Into<V::Prev>>(
        &'a self,
        vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>,
        prev: Prev,
    ) where
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.set_prev_of(self, prev.into())
    }

    /// Clears next references of the node.
    #[inline(always)]
    pub fn clear_next<'rf, P>(&'a self, vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.clear_next_of(self)
    }

    /// Clears previous references of the node.
    #[inline(always)]
    pub fn clear_prev<'rf, P>(&'a self, vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.clear_prev_of(self)
    }

    // with mut key - NodeDataLazyClose
    /// Closes the lazy node, takes out and returns its data.
    ///
    /// # Panics
    ///
    /// Panics if the node is already closed; i.e., if `self.is_closed()`.
    #[inline(always)]
    pub fn close_node_take_data<'rf, P>(&'a self, vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>) -> T
    where
        V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
        P: PinnedVec<Node<'a, V, T>> + 'a,
        SelfRefColMut<'rf, 'a, V, T, P>: Reclaim<V::Prev, V::Next>,
    {
        vec_mut.close_node_take_data(self)
    }

    /// Closes the lazy node, takes out and returns its data.
    /// Skips memory reclaim operation regardless of the utilization.
    ///
    /// # Panics
    ///
    /// Panics if the node is already closed; i.e., if `self.is_closed()`.
    #[inline(always)]
    pub fn close_node_take_data_no_reclaim<'rf, P>(
        &'a self,
        vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>,
    ) -> T
    where
        V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
        P: PinnedVec<Node<'a, V, T>> + 'a,
        SelfRefColMut<'rf, 'a, V, T, P>: Reclaim<V::Prev, V::Next>,
    {
        vec_mut.close_node_take_data_no_reclaim(self)
    }

    // with mut key - NodeRefsVec
    /// Returns a mutable reference to previous references of the node.
    #[inline(always)]
    pub fn prev_vec_mut<'rf, P>(
        &'a self,
        vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>,
    ) -> &mut Vec<&'a Node<'a, V, T>>
    where
        V: Variant<'a, T, Prev = NodeRefsVec<'a, V, T>>,
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.get_prev_vec_mut(self)
    }

    /// Returns a mutable reference to next references of the node.
    #[inline(always)]
    pub fn next_vec_mut<'rf, P>(
        &'a self,
        vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>,
    ) -> &mut Vec<&'a Node<'a, V, T>>
    where
        V: Variant<'a, T, Next = NodeRefsVec<'a, V, T>>,
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.get_next_vec_mut(self)
    }

    // with mut key - NodeRefsArray
    /// Returns a mutable reference to previous references of the node.
    #[inline(always)]
    pub fn prev_array_mut<'rf, P, const N: usize>(
        &'a self,
        vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>,
    ) -> &mut [Option<&'a Node<'a, V, T>>; N]
    where
        V: Variant<'a, T, Prev = NodeRefsArray<'a, N, V, T>>,
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.get_prev_array_mut(self)
    }

    /// Returns a mutable reference to next references of the node.
    #[inline(always)]
    pub fn next_array_mut<'rf, P, const N: usize>(
        &'a self,
        vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>,
    ) -> &mut [Option<&'a Node<'a, V, T>>; N]
    where
        V: Variant<'a, T, Next = NodeRefsArray<'a, N, V, T>>,
        P: PinnedVec<Node<'a, V, T>>,
    {
        vec_mut.get_next_array_mut(self)
    }

    // helpers - test
    /// Returns whether or not two node references are pointing to the same `Node`; i.e., checks referential equality.
    pub fn ref_eq(&self, other: &Self) -> bool {
        let left = self as *const Self;
        let right = other as *const Self;
        left == right
    }

    pub(crate) fn ref_eq_to_ptr(&self, other: *const Node<'a, V, T>) -> bool {
        let left = self as *const Self;
        left == other
    }
}

impl<'a, V, T> Clone for Node<'a, V, T>
where
    V: Variant<'a, T>,
    V::Storage: Clone,
    T: Clone,
{
    /// Clones the node data without the references.
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            prev: Default::default(),
            next: Default::default(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{
        MemoryReclaimOnThreshold, NodeData, NodeRefSingle, NodeRefs, NodeRefsArray, NodeRefsVec,
        SelfRefCol,
    };
    use orx_split_vec::prelude::PinnedVec;

    #[derive(Debug, Clone, Copy)]
    struct Var;
    impl<'a> Variant<'a, char> for Var {
        type Storage = NodeDataLazyClose<char>;
        type Prev = NodeRefSingle<'a, Self, char>;
        type Next = NodeRefsVec<'a, Self, char>;
        type Ends = NodeRefsArray<'a, 2, Self, char>;
        type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    }

    #[test]
    fn new_free_node() {
        let node = Node::<Var, _>::new_free_node('x');
        assert!(node.prev().get().is_none());
        assert!(node.next().get().is_empty());
        assert_eq!(Some(&'x'), node.data());
    }

    #[test]
    fn new() {
        let x = Node::<Var, _>::new_free_node('x');
        let y = Node::<Var, _>::new_free_node('y');
        let node = Node::<Var, _>::new(
            NodeDataLazyClose::active('z'),
            NodeRefSingle::new(Some(&x)),
            NodeRefsVec::new(vec![&y]),
        );

        assert!(node.prev().get().unwrap().ref_eq(&x));
        assert!(node.next().get()[0].ref_eq(&y));
        assert_eq!(Some(&'z'), node.data());
    }

    #[test]
    fn clone_does_not_clone_references() {
        let x = Node::<Var, _>::new_free_node('x');
        let y = Node::<Var, _>::new_free_node('y');
        let node = Node::<Var, _>::new(
            NodeDataLazyClose::active('z'),
            NodeRefSingle::new(Some(&x)),
            NodeRefsVec::new(vec![&y]),
        );

        let clone = node.clone(); // clone does not bring references!
        assert!(clone.prev().get().is_none());
        assert!(clone.next().get().is_empty());
        assert_eq!(Some(&'z'), clone.data());
    }

    #[test]
    fn data_mut() {
        let mut node = Node::<Var, _>::new_free_node('x');
        assert_eq!(Some(&'x'), node.data());

        *node.data_mut().unwrap() = 'y';
        assert_eq!(Some(&'y'), node.data());
    }

    #[test]
    fn swap_data() {
        let mut vec = SelfRefCol::<Var, _>::new();

        {
            let x = SelfRefColMut::new(&mut vec);
            let rf = x.push_get_ref('a');
            x.set_ends([Some(rf), None]);
        }

        assert_eq!(vec.pinned_vec.len(), 1);
        assert_eq!(vec.pinned_vec.get(0).unwrap().data().unwrap(), &'a');

        {
            let x = SelfRefColMut::new(&mut vec);
            let first = x.ends().get()[0].unwrap();
            let old_value = x.swap_data(first, 'z');
            assert_eq!('a', old_value);
        }

        assert_eq!(vec.pinned_vec.len(), 1);
        assert_eq!(vec.pinned_vec.get(0).unwrap().data().unwrap(), &'z');

        {
            let x = SelfRefColMut::new(&mut vec);
            let first = x.ends().get()[0].unwrap();
            let old_value = first.swap_data(&x, 'o');
            assert_eq!('z', old_value);
        }

        assert_eq!(vec.pinned_vec.len(), 1);
        assert_eq!(vec.pinned_vec.get(0).unwrap().data().unwrap(), &'o');
    }

    #[test]
    fn close_node_take_data() {
        let mut col = SelfRefCol::<Var, _>::new();
        assert!(col.pinned_vec.is_empty());

        {
            let x = SelfRefColMut::new(&mut col);
            let a = x.push_get_ref('a');
            assert!(a.is_active());

            let val_a = a.close_node_take_data(&x);
            assert_eq!('a', val_a);
            assert!(a.is_closed());
        }
    }

    #[test]
    fn close_node_take_data_no_reclaim() {
        // will reclaim
        let mut col = SelfRefCol::<Var, _>::new();
        let a = col.mutate_take((), |x, _| {
            let a = x.push_get_ref('a');
            x.push_get_ref('b');
            x.push_get_ref('c');
            a.index(&x)
        });

        let memory_state = col.memory_reclaim_policy.0.id;

        let val_a = col.mutate_take(a, |x, a| x.as_node_ref(a).close_node_take_data(&x));
        assert_eq!(val_a, 'a');

        assert_ne!(col.memory_reclaim_policy.0.id, memory_state);

        // will not reclaim
        let mut col = SelfRefCol::<Var, _>::new();
        let a = col.mutate_take((), |x, _| {
            let a = x.push_get_ref('a');
            x.push_get_ref('b');
            x.push_get_ref('c');
            a.index(&x)
        });

        let memory_state = col.memory_reclaim_policy.0.id;

        let val_a = col.mutate_take(a, |x, a| {
            x.as_node_ref(a).close_node_take_data_no_reclaim(&x)
        });
        assert_eq!(val_a, 'a');

        assert_eq!(col.memory_reclaim_policy.0.id, memory_state);
    }

    #[test]
    #[should_panic]
    fn close_node_take_data_on_closed_node() {
        let mut col = SelfRefCol::<Var, _>::new();
        let x = SelfRefColMut::new(&mut col);
        let a = x.push_get_ref('a');
        assert!(a.is_active());

        let val_a = a.close_node_take_data(&x);
        assert_eq!('a', val_a);
        assert!(a.is_closed());

        let _ = a.close_node_take_data(&x); // panics!
    }

    #[test]
    fn set_clear_prev_next_single() {
        #[derive(Debug, Clone, Copy)]
        struct Var;
        impl<'a> Variant<'a, char> for Var {
            type Storage = NodeDataLazyClose<char>;
            type Prev = NodeRefSingle<'a, Self, char>;
            type Next = NodeRefSingle<'a, Self, char>;
            type Ends = NodeRefsArray<'a, 2, Self, char>;
            type MemoryReclaim = MemoryReclaimOnThreshold<2>;
        }

        let mut col = SelfRefCol::<Var, _>::new();
        assert!(col.pinned_vec.is_empty());

        {
            let x = SelfRefColMut::new(&mut col);
            let a = x.push_get_ref('a');
            let b = x.push_get_ref('b');

            a.set_prev_refs(&x, b.into());
            assert!(a.prev().get().unwrap().ref_eq(b));

            a.set_prev(&x, b);
            assert!(a.prev().get().unwrap().ref_eq(b));

            a.set_prev(&x, b);
            assert!(a.prev().get().unwrap().ref_eq(b));

            a.clear_prev(&x);
            assert!(a.prev().get().is_none());

            a.set_prev_refs(&x, NodeRefSingle::empty());
            assert!(a.prev().get().is_none());

            b.set_next(&x, a);
            assert!(b.next().get().unwrap().ref_eq(a));

            b.set_next(&x, a);
            assert!(b.next().get().unwrap().ref_eq(a));

            b.clear_next(&x);
            assert!(b.next().get().is_none());

            a.set_next(&x, b);
            b.set_prev(&x, a);
        }

        let a = &col.pinned_vec[0];
        let b = &col.pinned_vec[1];
        assert!(a.next().get().unwrap().ref_eq(b));
        assert!(b.prev().get().unwrap().ref_eq(a));
    }

    #[test]
    fn set_clear_prev_next_vec() {
        #[derive(Debug, Clone, Copy)]
        struct Var;
        impl<'a> Variant<'a, char> for Var {
            type Storage = NodeDataLazyClose<char>;
            type Prev = NodeRefsVec<'a, Self, char>;
            type Next = NodeRefsVec<'a, Self, char>;
            type Ends = NodeRefsArray<'a, 2, Self, char>;
            type MemoryReclaim = MemoryReclaimOnThreshold<2>;
        }

        let mut vec = SelfRefCol::<Var, _>::new();
        assert!(vec.pinned_vec.is_empty());

        {
            let x = SelfRefColMut::new(&mut vec);
            let a = x.push_get_ref('a');
            let b = x.push_get_ref('b');
            let c = x.push_get_ref('c');

            a.set_next_refs(&x, NodeRefsVec::new(vec![b]));
            assert_eq!(a.next().get().len(), 1);
            assert!(a.next().get().first().unwrap().ref_eq(b));

            a.set_next(&x, vec![b]);
            assert_eq!(a.next().get().len(), 1);
            assert!(a.next().get().first().unwrap().ref_eq(b));

            a.next_vec_mut(&x).push(c);

            assert_eq!(a.next().get().len(), 2);
            assert!(a.next().get().first().unwrap().ref_eq(b));
            assert!(a.next().get().get(1).unwrap().ref_eq(c));

            a.clear_next(&x);
            assert!(a.next().get().is_empty());

            a.set_next(&x, NodeRefsVec::empty());
            assert!(a.next().get().is_empty());

            b.set_prev(&x, vec![a]);
            assert_eq!(b.prev().get().len(), 1);
            assert!(b.prev().get().first().unwrap().ref_eq(a));

            c.set_prev(&x, vec![a, b]);
            assert_eq!(c.prev().get().len(), 2);
            assert!(c.prev().get().first().unwrap().ref_eq(a));
            assert!(c.prev().get().get(1).unwrap().ref_eq(b));

            c.prev_vec_mut(&x).remove(0);
            assert_eq!(c.prev().get().len(), 1);
            assert!(c.prev().get().first().unwrap().ref_eq(b));

            c.clear_prev(&x);
            assert!(c.prev().get().is_empty());
        }
    }

    #[test]
    fn set_clear_prev_next_array() {
        #[derive(Debug, Clone, Copy)]
        struct Var;
        impl<'a> Variant<'a, char> for Var {
            type Storage = NodeDataLazyClose<char>;
            type Prev = NodeRefsArray<'a, 1, Self, char>;
            type Next = NodeRefsArray<'a, 2, Self, char>;
            type Ends = NodeRefsArray<'a, 2, Self, char>;
            type MemoryReclaim = MemoryReclaimOnThreshold<2>;
        }

        let mut vec = SelfRefCol::<Var, _>::new();
        assert!(vec.pinned_vec.is_empty());

        {
            let x = SelfRefColMut::new(&mut vec);
            let a = x.push_get_ref('a');
            let b = x.push_get_ref('b');
            let c = x.push_get_ref('c');

            a.set_next_refs(&x, NodeRefsArray::new([Some(b), Some(c)]));
            assert!(a.next().get()[0].unwrap().ref_eq(b));
            assert!(a.next().get()[1].unwrap().ref_eq(c));

            a.set_next(&x, [Some(b), None]);
            assert!(a.next().get()[0].unwrap().ref_eq(b));
            assert!(a.next().get()[1].is_none());

            a.next_array_mut(&x)[1] = Some(c);
            assert!(a.next().get()[1].unwrap().ref_eq(c));

            a.clear_next(&x);
            assert!(a.next().get()[0].is_none());
            assert!(a.next().get()[1].is_none());

            a.set_next(&x, NodeRefsArray::empty());
            assert!(a.next().get()[0].is_none());
            assert!(a.next().get()[1].is_none());

            b.set_prev_refs(&x, NodeRefsArray::new([Some(a)]));
            assert!(b.prev().get()[0].unwrap().ref_eq(a));

            b.prev_array_mut(&x)[0] = None;
            assert!(b.prev().get()[0].is_none());

            b.set_prev(&x, [Some(c)]);
            assert!(b.prev().get()[0].unwrap().ref_eq(c));

            b.clear_prev(&x);
            assert!(b.prev().get()[0].is_none());
        }
    }
}
