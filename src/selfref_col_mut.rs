use crate::{
    nodes::index::NodeIndex,
    variants::memory_reclaim::{MemoryReclaimPolicy, Reclaim},
    Node, NodeData, NodeDataLazyClose, NodeIndexError, NodeRefs, NodeRefsArray, NodeRefsVec,
    SelfRefCol, Variant,
};
use orx_split_vec::prelude::PinnedVec;
use std::ops::Deref;

/// Struct allowing to safely, conveniently and efficiently mutate a self referential collection.
///
/// # Safety
///
/// This struct holds a mutable reference to the underlying self referential collection.
/// Therefore, it allows to mutate the vector with immutable references, as in internal mutability of a refcell.
///
/// This struct cannot be created externally.
/// It is only constructed by `SelfRefCol`s methods such as `mutate` and `mutate_take` methods.
/// You may find corresponding safety guarantees in the documentation of these methods, which in brief, relies on careful encapsulation of mutation
/// preventing any reference to leak into the collection or any node reference to leak out.
///
/// # Convenience
///
/// The nodes of the self referential collection, `SelfRefNode`, has convenient methods to mutate the references, such as:
///
/// * `set_prev`
/// * `set_next`
/// * `clear_next`
/// * `clear_prev`
/// * `close_node_take_data`
///
/// In addition, `SelfRefColMut` provides the following vector methods:
///
/// * `push_get_ref`
/// * `set_ends`
///
/// These allow to easily and conveniently update the relations among the nodes without lifetime or ownership complexities.
///
/// # Example
///
/// For instance, the following example illustrates the `push_front` method in a doubly linked list.
/// The lambda argument `x` is a `SelfRefColMut`.
/// You might notice that the lambda body is close enough to the pseudocode of the method.
/// All complexity of the safety guarantees is hidden by the additional parameter or the key, `x`, passed into these mut methods.
///
/// ```rust ignore
/// pub fn push_front(&mut self, value: T) {
///     self.vec
///         .move_mutate(value, |x, value| match x.ends().front() {
///             None => {
///                 let node = x.push_get_ref(value);
///                 x.set_ends([Some(node), Some(node)]);
///             }
///             Some(prior_front) => {
///                 let new_front = x.push_get_ref(value);
///                 new_front.set_next(&x, prior_front);
///                 prior_front.set_prev(&x, new_front);
///                 x.set_ends([Some(new_front), x.ends().back()]);
///             }
///         });
///     self.len += 1;
/// }
/// ```
pub struct SelfRefColMut<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
    'a: 'rf,
{
    pub(crate) col: &'rf mut SelfRefCol<'a, V, T, P>,
}

impl<'rf, 'a, V, T, P> SelfRefColMut<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    pub(crate) fn new(vec: &'rf mut SelfRefCol<'a, V, T, P>) -> Self {
        Self { col: vec }
    }

    #[inline(always)]
    pub(crate) fn set_next_of(&self, node: &'a Node<'a, V, T>, next: V::Next) {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.next = next;
    }

    #[inline(always)]
    pub(crate) fn set_prev_of(&self, node: &'a Node<'a, V, T>, prev: V::Prev) {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.prev = prev;
    }

    #[inline(always)]
    pub(crate) fn clear_next_of(&self, node: &'a Node<'a, V, T>) {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.next = V::Next::default();
    }

    #[inline(always)]
    pub(crate) fn clear_prev_of(&self, node: &'a Node<'a, V, T>) {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.prev = V::Prev::default();
    }

    #[inline(always)]
    pub(crate) fn swap_data(&self, node: &'a Node<'a, V, T>, new_value: T) -> T {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.data.swap_data(new_value)
    }

    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    fn ends_mut(&self) -> &mut V::Ends {
        unsafe { into_mut(&self.col.ends) }
    }

    // index
    /// ***O(1)*** Converts the `node_index` to `Some` of the valid reference to the node in this collection.
    ///
    /// If the node index is invalid, the method returns `None`.
    ///
    /// Note that the validity of the node index can also be queried by `node_index::is_valid_for_collection` method.
    ///
    /// `get_node_ref(collection)` returns `Some` if all of of the following safety and correctness conditions hold:
    /// * this index is created from the given `collection`,
    /// * the node this index is created for still belongs to the `collection`; i.e., is not removed,
    /// * the node positions in the `collection` are not reorganized to reclaim memory.
    #[inline(always)]
    pub fn get_node_ref(&self, node_index: NodeIndex<'a, V, T>) -> Option<&'a Node<'a, V, T>> {
        match node_index.is_valid_for_collection(self.col) {
            true => Some(node_index.node_key),
            false => None,
        }
    }

    /// ***O(1)*** Converts the `node_index` to a `Ok` of the valid reference to the node in this collection.
    ///
    /// If the node index is invalid, the method returns `Err` of the corresponding `NodeIndexError` depending on the reason of invalidity.
    ///
    /// Note that the corresponding error can also be queried by `node_index::invalidity_reason_for_collection` method.
    ///
    /// `get_node_ref_or_error(collection)` returns `Ok` if all of of the following safety and correctness conditions hold:
    /// * this index is created from the given `collection`,
    /// * the node this index is created for still belongs to the `collection`; i.e., is not removed,
    /// * the node positions in the `collection` are not reorganized to reclaim memory.
    #[inline(always)]
    pub fn get_node_ref_or_error(
        &self,
        node_index: NodeIndex<'a, V, T>,
    ) -> Result<&'a Node<'a, V, T>, NodeIndexError>
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        match node_index.invalidity_reason_for_collection(self.col) {
            None => Ok(node_index.node_key),
            Some(error) => Err(error),
        }
    }

    /// ***O(1)*** Converts the `node_index` to a reference to the node in this collection.
    /// The call panics if `node_index.is_valid_for_collection(collection)` is false; i.e., if this node index is not valid for this collection.
    ///
    /// # Panics
    ///
    /// Panics if the node index is invalid; i.e., if `node_index.is_valid_for_collection` returns false.
    ///
    /// Note that `is_valid_for_collection` returns true if all of of the following safety and correctness conditions hold:
    /// * this index is created from the given `collection`,
    /// * the node this index is created for still belongs to the `collection`; i.e., is not removed,
    /// * the node positions in the `collection` are not reorganized to reclaim memory.
    #[inline(always)]
    pub fn as_node_ref(&self, node_index: NodeIndex<'a, V, T>) -> &'a Node<'a, V, T> {
        assert!(node_index.is_valid_for_collection(self.col));
        node_index.node_key
    }

    // nodes
    /// Returns a reference to the first node of the collection.
    pub fn first_node<'b>(&self) -> Option<&'b Node<'a, V, T>> {
        self.col.pinned_vec.first().map(|x| unsafe { into_ref(x) })
    }

    /// Returns a reference to the last node of the collection.
    pub fn last_node<'b>(&self) -> Option<&'b Node<'a, V, T>> {
        self.col.pinned_vec.last().map(|x| unsafe { into_ref(x) })
    }

    /// Returns a reference to the `at`-th node of the collection.
    pub fn get_node<'b>(&self, at: usize) -> Option<&'b Node<'a, V, T>> {
        self.col.pinned_vec.get(at).map(|x| unsafe { into_ref(x) })
    }

    // NodeRefsVec
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_prev_vec_mut<'b>(
        &self,
        node: &'a Node<'a, V, T>,
    ) -> &'b mut Vec<&'a Node<'a, V, T>>
    where
        V: Variant<'a, T, Prev = NodeRefsVec<'a, V, T>>,
    {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.prev.get_mut()
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_next_vec_mut<'b>(
        &self,
        node: &'a Node<'a, V, T>,
    ) -> &'b mut Vec<&'a Node<'a, V, T>>
    where
        V: Variant<'a, T, Next = NodeRefsVec<'a, V, T>>,
    {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.next.get_mut()
    }

    // NodeRefsArray
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_prev_array_mut<'b, const N: usize>(
        &self,
        node: &'a Node<'a, V, T>,
    ) -> &'b mut [Option<&'a Node<'a, V, T>>; N]
    where
        V: Variant<'a, T, Prev = NodeRefsArray<'a, N, V, T>>,
    {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.prev.get_mut()
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_next_array_mut<'b, const N: usize>(
        &self,
        node: &'a Node<'a, V, T>,
    ) -> &'b mut [Option<&'a Node<'a, V, T>>; N]
    where
        V: Variant<'a, T, Next = NodeRefsArray<'a, N, V, T>>,
    {
        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.next.get_mut()
    }

    // pub
    /// Pushes the `value` to the vector and returns a reference to the created node.
    ///
    /// # Example
    ///
    /// The following code block demonstrates the push-front operation of a singly linked list.
    /// The code is branched depending on whether or not there already exists a front; i.e., the list is not empty.
    /// In either case,
    /// * the new value is pushed to the self referential collection;
    /// * and a reference to the new list node containing this value is received right after by the `push_get_ref` method;
    /// * this reference is then used to establish singly linked list relations.
    ///
    /// ```rust ignore
    /// pub fn push_front(&mut self, value: T) {
    ///     self.col
    ///         .move_mutate(value, |x, value| match x.ends().front() {
    ///             Some(prior_front) => {
    ///                 let new_front = x.push_get_ref(value);
    ///                 new_front.set_next(&x, prior_front);
    ///                 x.set_ends(new_front);
    ///             }
    ///             None => {
    ///                 let node = x.push_get_ref(value);
    ///                 x.set_ends(node);
    ///             }
    ///         });
    /// }
    /// ```
    pub fn push_get_ref<'b>(&self, value: T) -> &'b Node<'a, V, T> {
        let node = Node::new_free_node(value);
        let vec = unsafe { into_mut(self.col) };
        vec.pinned_vec.push(node);
        vec.len += 1;
        unsafe { into_ref(self.col.pinned_vec.last_unchecked()) }
    }

    /// Sets the ends of the self referential collection to the given `ends`.
    ///
    /// Ends represent special references of the self referential structure.
    /// It can be nothing; i.e., `NodeRefNone`; however, they are common in such structures.
    /// For instance,
    /// * ends of a singly linked list is the **front** of the list which can be represented as a `NodeRefSingle` reference;
    /// * ends of a doubly linked list contains two references, **front** and **back** of the list which can be represented by a `NodeRefsArray<2, _, _>`;
    /// * ends of a tree is the **root** which can again be represented as a `NodeRefSingle` reference.
    ///
    /// Ends of a `SelfRefCol` is generic over `NodeRefs` trait which can be decided on the structure's requirement.
    #[inline(always)]
    pub fn set_ends_refs(&self, ends: V::Ends) {
        *self.ends_mut() = ends;
    }

    /// Sets the ends of the self referential collection to the given `ends`.
    ///
    /// Ends represent special references of the self referential structure.
    /// It can be nothing; i.e., `NodeRefNone`; however, they are common in such structures.
    /// For instance,
    /// * ends of a singly linked list is the **front** of the list which can be represented as a `NodeRefSingle` reference;
    /// * ends of a doubly linked list contains two references, **front** and **back** of the list which can be represented by a `NodeRefsArray<2, _, _>`;
    /// * ends of a tree is the **root** which can again be represented as a `NodeRefSingle` reference.
    ///
    /// Ends of a `SelfRefCol` is generic over `NodeRefs` trait which can be decided on the structure's requirement.
    #[inline(always)]
    pub fn set_ends<Ends: Into<V::Ends>>(&self, ends: Ends) {
        *self.ends_mut() = ends.into();
    }
}

impl<'rf, 'a, V, T, P> SelfRefColMut<'rf, 'a, V, T, P>
where
    V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
    P: PinnedVec<Node<'a, V, T>> + 'a,
    SelfRefColMut<'rf, 'a, V, T, P>: Reclaim<V::Prev, V::Next>,
{
    pub(crate) fn close_node_take_data_no_reclaim(&self, node: &'a Node<'a, V, T>) -> T {
        debug_assert!(node.data.is_active());
        let vec_mut = unsafe { into_mut(self) };
        vec_mut.col.len -= 1;

        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.prev = V::Prev::empty();
        node.next = V::Next::empty();
        node.data.close().expect("node is active")
    }

    pub(crate) fn close_node_take_data(&self, node: &'a Node<'a, V, T>) -> T {
        debug_assert!(node.data.is_active());
        let vec_mut = unsafe { into_mut(self) };
        vec_mut.col.len -= 1;

        let node = std::hint::black_box(unsafe { into_mut(node) });
        node.prev = V::Prev::empty();
        node.next = V::Next::empty();
        let data = node.data.close().expect("node is active");
        V::MemoryReclaim::reclaim_closed_nodes(vec_mut);
        data
    }
}

impl<'rf, 'a, V, T, P> Deref for SelfRefColMut<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    type Target = SelfRefCol<'a, V, T, P>;

    fn deref(&self) -> &Self::Target {
        self.col
    }
}

#[allow(invalid_reference_casting)]
#[inline(always)]
pub(crate) unsafe fn into_mut<'a, T>(reference: &T) -> &'a mut T {
    &mut *(reference as *const T as *mut T)
}

#[inline(always)]
pub(crate) unsafe fn into_ref<'a, T>(reference: &T) -> &'a T {
    &*(reference as *const T)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{MemoryReclaimNever, NodeRefSingle, NodeRefs, NodeRefsArray, NodeRefsVec};

    #[derive(Debug, Clone, Copy)]
    struct Var;
    impl<'a> Variant<'a, char> for Var {
        type Storage = NodeDataLazyClose<char>;
        type Prev = NodeRefSingle<'a, Self, char>;
        type Next = NodeRefsVec<'a, Self, char>;
        type Ends = NodeRefsArray<'a, 2, Self, char>;
        type MemoryReclaim = MemoryReclaimNever;
    }

    #[test]
    fn push_get_ref() {
        let mut vec = SelfRefCol::<Var, _>::new();
        assert!(vec.pinned_vec.is_empty());

        {
            let x = SelfRefColMut::new(&mut vec);

            let a = x.push_get_ref('a'); // 'a' cannot leak the scope

            assert_eq!(vec.pinned_vec.len(), 1);

            assert!(a.ref_eq(&vec.pinned_vec[0]));

            assert_eq!(vec.pinned_vec[0].data().unwrap(), &'a');
            assert!(vec.pinned_vec[0].prev().get().is_none());
            assert!(vec.pinned_vec[0].next().get().is_empty());
        }

        for _ in 1..1000 {
            vec.pinned_vec.push(Node::new_free_node('b'));
        }

        assert_eq!(vec.pinned_vec[0].data().unwrap(), &'a');
    }

    #[test]
    fn set_ends() {
        let mut vec = SelfRefCol::<Var, _>::new();
        assert!(vec.pinned_vec.is_empty());
        assert!(vec.ends().get()[0].is_none());
        assert!(vec.ends().get()[1].is_none());

        {
            let x = SelfRefColMut::new(&mut vec);

            let a = x.push_get_ref('a');
            let b = x.push_get_ref('b');

            x.set_ends_refs(NodeRefsArray::new([Some(b), Some(a)]));

            assert!(b.ref_eq(x.ends().get()[0].unwrap()));
            assert!(a.ref_eq(x.ends().get()[1].unwrap()));

            x.set_ends([Some(a), Some(b)]);

            assert!(a.ref_eq(x.ends().get()[0].unwrap()));
            assert!(b.ref_eq(x.ends().get()[1].unwrap()));
        }

        for _ in 1..1000 {
            vec.pinned_vec.push(Node::new_free_node('b'));
        }

        assert_eq!(Some(&'a'), vec.ends().get()[0].unwrap().data());
        assert_eq!(Some(&'b'), vec.ends().get()[1].unwrap().data());
    }
}
