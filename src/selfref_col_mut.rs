use std::ops::Deref;

use crate::{
    variants::memory_reclaim::MemoryReclaimPolicy, Node, NodeData, NodeDataLazyClose, NodeRefs,
    NodeRefsArray, NodeRefsVec, SelfRefCol, Variant,
};
use orx_split_vec::{prelude::PinnedVec, SplitVec};

/// Struct allowing to safely, conveniently and efficiently mutate a self referential collection.
///
/// # Safety
///
/// This struct holds a mutable reference to the underlying self referential collection.
/// Therefore, it allows to mutate the vector with immutable references, as in internal mutability of a refcell.
///
/// This struct cannot be created externally.
/// It is only constructed by `SelfRefCol`s `move_mutate` and `mutate_take` methods.
///
/// ## `move_mutate`
///
/// Move-mutate accepts a lambda having an access to `SelfRefColMut` to mutate the vector and an additional value which is moved into the lambda.
/// The method does not return a value.
///
/// Note that the lambda is a function pointer rather than a closure.
/// Therefore,
/// * a reference external to the vector cannot be leaked in;
/// * a reference to an element of the vector cannot leak out.
///
/// Thus the compactness of the self referential collection is preserved.
///
/// ## `mutate_take`
///
/// Mutate-take takes a single parameter, a lambda having an access to `SelfRefColMut` to mutate the vector.
/// The method returns a value of element type `T`.
/// In other words, it takes out a value from the vector and returns it.
///
/// Note that the lambda is a function pointer rather than a closure.
/// Therefore,
/// * a reference external to the vector cannot be leaked in;
/// * a reference to an element of the vector cannot leak out, note that the return type is strictly an owned value of the element type.
///
/// Thus the compactness of the self referential collection is preserved.
///
/// Mutations of the references of the self referential collection are conveniently handled by methods of `SelfRefNode`.
/// However, all these methods require a reference to a `SelfRefColMut`.
/// In other words, `SelfRefColMut` is the key to enable these mutations.
/// The safety is then guaranteed by the following:
/// * `SelfRefColMut` is never explicitly constructed by the caller.
/// The caller only provides definition of the mutations in the form of a lambda, which is specifically a `fn` as explained above.
/// * `SelfRefCol` methods `move_mutate` and `mutate_take` require a mutable reference to the vector;
/// and hence, there might be only at most one `SelfRefColMut` instance at any given time.
/// * Inside the lambda, however, multiple mutations are easily enabled and allowed with the guarantees of the encapsulation.
///
/// ## `move_mutate_take`
///
/// As the name suggests, this method is the combination of the prior two:
/// * we move a value to the non-capturing lambda,
/// * we mutate references and the collection encapsulated inside the lambda using the key,
/// * we take one element out.
///
/// This method is most suitable for operations involving swaps.
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
pub struct SelfRefColMut<'rf, 'a, V, T, P = SplitVec<Node<'a, V, T>>>
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

    // mem
    /// Manually attempts to reclaim closed nodes.
    pub fn reclaim_closed_nodes(&self)
    where
        P: 'a,
    {
        let vec_mut = unsafe { into_mut(self) };
        V::MemoryReclaim::reclaim_closed_nodes(vec_mut);
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
