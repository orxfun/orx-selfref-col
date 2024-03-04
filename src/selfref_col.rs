use crate::{
    nodes::{can_leak::CanLeak, node::Node},
    selfref_col_mut::{into_mut, SelfRefColMut},
    variants::{
        memory_reclaim::{MemoryReclaimAlways, MemoryReclaimPolicy},
        variant::Variant,
    },
    MemoryReclaimOnThreshold,
};
use orx_split_vec::{prelude::PinnedVec, Recursive, SplitVec};
use std::marker::PhantomData;

/// `SelfRefCol` is a core data structure to conveniently build safe and efficient self referential collections, such as linked lists and trees.
///
/// Note that this core structure is capable of representing a wide range of self referential collections, where the variant is conveniently defined by expressive trait type definitions.
///
/// The represented collections have the following features:
/// * Relations are represented by regular `&` references avoiding the need to use smart pointers such as `Box`, `Rc`, `Arc`, etc.
/// * The collection makes sure that these references are set only among elements of the collections.
/// In other words, no external references are allowed, or references of elements of the collection cannot leak out.
/// This constructs the safety guarantees.
/// * The elements of the collection are internally stored in a `PinnedVec` implementation, which is crucial for the correctness of the references.
/// Furthermore, in this way, elements of the collection are stored close to each other rather than being in arbitrary locations in memory.
/// This provides better cache locality when compared to such collections where elements are stored by arbitrary heap allocations.
///
/// The collection is defined by the following generic arguments:
/// * `T`: type of the elements stored in the collection.
/// * `V`: type of the `Variant` defining the structure of the collection with the following:
///   * `V::Storage`: defines how the elements of `T` will be stored:
///     * `NodeDataLazyClose`: elements are stored as `Option<T>` allowing lazy node closure or element removal;
///     * `NodeDataEagerClose`: elements are stored directly as `T`.
///   * `V::Prev`: defines how references to previous elements will be stored.
///     * `NodeRefNone`: there is no previous reference of elements.
///     * `NodeRefSingle`: there is either one or no previous reference of elements, stored as `Option<&Node>`.
///     * `NodeRefsArray`: there are multiple possible previous references up to a constant number `N`, stored as `[Option<&Node>; N]`.
///     * `NodeRefsVec`: there are multiple possible previous references, stored as `Vec<&Node>`.
///   * `V::Next`: defines how references to next elements will be stored:
///     * Similarly, represented as either one of `NodeRefNone` or `NodeRefSingle` or `NodeRefsArray` or `NodeRefsVec`.
///   * `V::Ends`: defines how references to ends of the collection will be stored:
///     * Similarly, represented as either one of `NodeRefNone` or `NodeRefSingle` or `NodeRefsArray` or `NodeRefsVec`.
///   * `V::MemoryReclaim`: defines how memory of closed nodes will be reclaimed:
///     * `MemoryReclaimNever` will never claim closed nodes.
///     * `MemoryReclaimOnThreshold<D>` will claim memory of closed nodes whenever the ratio of closed nodes exceeds one over `2^D`.
///
/// # Example
///
/// Consider the following four structs implementing `Variant` to define four different self referential collections.
/// Note that the definitions are expressive and concise leading to efficient implementations.
///
/// ```rust
/// use orx_selfref_col::*;
///
/// #[derive(Clone, Copy)]
/// struct SinglyListVariant;
///
/// impl<'a, T: 'a> Variant<'a, T> for SinglyListVariant {
///     type Storage = NodeDataLazyClose<T>; // lazy close
///     type MemoryReclaim = MemoryReclaimOnThreshold<2>; // closed nodes will be reclaimed when utilization drops below 75%
///     type Prev = NodeRefNone; // previous nodes are not stored
///     type Next = NodeRefSingle<'a, Self, T>; // there is only one next node, if any
///     type Ends = NodeRefSingle<'a, Self, T>; // there is only one end, namely the front of the list
/// }
///
/// #[derive(Clone, Copy)]
/// struct DoublyListVariant;
///
/// impl<'a, T: 'a> Variant<'a, T> for DoublyListVariant {
///     type Storage = NodeDataLazyClose<T>; // lazy close
///     type MemoryReclaim = MemoryReclaimOnThreshold<3>; // closed nodes will be reclaimed when utilization drops below 87.5%
///     type Prev = NodeRefSingle<'a, Self, T>; // there is only one previous node, if any
///     type Next = NodeRefSingle<'a, Self, T>; // there is only one next node, if any
///     type Ends = NodeRefsArray<'a, 2, Self, T>; // there are two ends, namely the front and back of the list
/// }
///
/// #[derive(Clone, Copy)]
/// struct BinaryTreeVariant;
///
/// impl<'a, T: 'a> Variant<'a, T> for BinaryTreeVariant {
///     type Storage = NodeDataLazyClose<T>; // lazy close
///     type MemoryReclaim = MemoryReclaimOnThreshold<1>; // closed nodes will be reclaimed when utilization drops below 50%
///     type Prev = NodeRefSingle<'a, Self, T>; // there is only one previous node, namely parent node, if any
///     type Next = NodeRefsArray<'a, 2, Self, T>; // there are 0, 1 or 2 next or children nodes
///     type Ends = NodeRefSingle<'a, Self, T>; // there is only one end, namely the root of the tree
/// }
///
/// #[derive(Clone, Copy)]
/// struct DynamicTreeVariant;
///
/// impl<'a, T: 'a> Variant<'a, T> for DynamicTreeVariant {
///     type Storage = NodeDataLazyClose<T>; // lazy close
///     type MemoryReclaim = MemoryReclaimNever; // closed nodes will be left as holes
///     type Prev = NodeRefSingle<'a, Self, T>; // there is only one previous node, namely parent node, if any
///     type Next = NodeRefsVec<'a, Self, T>; // there might be any number of next nodes, namely children nodes
///     type Ends = NodeRefSingle<'a, Self, T>; // there is only one end, namely the root of the tree
/// }
/// ```
///
/// # Crates using `SelfRefCol`
///
/// The following crates use `SelfRefCol` to conveniently build the corresponding data structure:
/// * [https://crates.io/crates/orx-linked-list](https://crates.io/crates/orx-linked-list): implements singly and doubly linked lists.
pub struct SelfRefCol<'a, V, T, P = SplitVec<Node<'a, V, T>, Recursive>>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    pub(crate) ends: V::Ends,
    pub(crate) pinned_vec: P,
    pub(crate) len: usize,
    pub(crate) memory_reclaim_policy: V::MemoryReclaim,
    pub(crate) phantom: PhantomData<&'a V>,
}

impl<'a, V, T, P> SelfRefCol<'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    /// Creates a new empty self referential collection.
    pub fn new() -> Self
    where
        P: Default,
    {
        Self {
            ends: V::Ends::default(),
            pinned_vec: Default::default(),
            len: 0,
            memory_reclaim_policy: Default::default(),
            phantom: Default::default(),
        }
    }

    /// Returns a reference to the ends of the self referential collection.
    ///
    /// Ends represent special references of the self referential structure.
    /// It can be nothing; i.e., `NodeRefNone`; however, they are common in such structures.
    /// For instance,
    /// * ends of a singly linked list is the **front** of the list which can be represented as a `NodeRefSingle` reference;
    /// * ends of a doubly linked list contains two references, **front** and **back** of the list which can be represented by a `NodeRefsArray<2, _, _>`;
    /// * ends of a tree is the **root** which can again be represented as a `NodeRefSingle` reference.
    ///
    /// Ends of a `SelfRefCol` is generic over `NodeRefs` trait which can be decided on the structure's requirement.
    pub fn ends(&self) -> &V::Ends {
        &self.ends
    }

    /// Returns length of the self referential collection.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether or not the self referential collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // mut
    /// Clears the collection: clears all elements and the ends of the collection.
    pub fn clear(&mut self) {
        self.ends = V::Ends::default();
        self.pinned_vec.clear();
        self.len = 0;
    }

    /// Manually attempts to reclaim closed nodes.
    ///
    /// # Safety
    ///
    /// Note that reclaiming closed nodes invalidates node indices (`NodeIndex`) which are already stored outside of this collection.
    ///
    /// * when `MemoryReclaim` policy is set to **`MemoryReclaimOnThreshold`**, there is no safety concern:
    ///   * In this policy, memory reclaim operations automatically happen whenever utilization is below a threshold.
    ///   * Therefore, manually triggering the reclaim operation is no different.
    ///   * `NodeIndex` compares the memory state of the collection and:
    ///     * `NodeIndex::get_ref` returns `None` in such a case, or
    ///     * `NodeIndex::as_ref` panics (equivalent to unwrapping the result of `get_ref`).
    /// * when `MemoryReclaim` policy is set to **`MemoryReclaimNever`**; however, the caller takes responsibility:
    ///   * In this policy, memory reclaim operations never happen implicitly.
    ///   * This can be considered as a performance optimization for cases where removals are rare.
    ///   * This further gives the luxury to keep size of the `NodeIndex` equal to one pointer (rather than two pointers as in the `MemoryReclaimOnThreshold` case).
    ///   * However, this means that `NodeIndex` is not able to detect the memory reclaims. The safety rule is then as follows:
    ///     * Node indices will never be invalid due to implicit memory reclaim operations.
    ///     * Node indices will be invalidated and must not be used whenever `SelfRefCol::reclaim_closed_nodes(&mut self)` is manually called.
    ///   * Importantly, note that this will not lead to UB.
    ///     * The safety concern is more around correctness of the reference rather than memory read violations.
    ///     * `SelfRefCol` will never allow to read outside its memory.
    pub fn reclaim_closed_nodes(&mut self)
    where
        P: 'a,
        MemoryReclaimOnThreshold<32>:
            MemoryReclaimPolicy<'a, V, T, <V as Variant<'a, T>>::Prev, <V as Variant<'a, T>>::Next>,
    {
        let mut vecmut = SelfRefColMut::new(self);
        MemoryReclaimAlways::reclaim_closed_nodes(&mut vecmut);
    }

    /// Method allowing to mutate the collection.
    ///
    /// This method takes the following arguments:
    /// * **`value_to_move`** is, as the name suggests, a value to be moved to the mutation lambda.
    /// Two common use cases to move this value to the lambda are:
    ///   * to add moved element(s) of `T` to the self referential collection,
    ///   * to use moved `NodeIndex` (indices) to access elements in constant time.
    /// * **`move_mutate_lambda`** is the expression defining the mutation.
    ///   * The lambda takes two parameters:
    ///     * the first parameter is the `SelfRefColMut` type which is the key for all `SelfRefNode` mutation and constant-time access methods;
    ///     * the second parameter is the value moved into the lambda, which is exactly the `value_to_move` parameter of this method.
    ///   * Note that the lambda is of a function pointer type; i.e., `fn`, rather than a function trait such as `FnOnce`.
    /// This is intentional and critical for safety guarantees of the collection.
    /// This prevents capturing data from the environment, as well as, prevents leaking references outside of the lambda.
    ///
    /// This design allows to conveniently mutate the references within the vector without the complexity of lifetimes and borrow checker.
    /// Prior references can be broken, references can be rearranged or new references can be built easily.
    /// Furthermore, references by `NodeIndex` can be used for direct constant-time access to elements.
    /// This convenience is achieved by the encapsulation of all mutations within a non-capturing lambda.
    ///
    /// # Example
    ///
    /// The following code block demonstrates the use of the `mutate` function to define the push-front method of a singly linked list.
    /// Note that `self.col` below is a `SelfRefCol`.
    /// The pushed `value` is moved to the lambda.
    /// Inside the lambda, this value is pushed to the list, which is stored inside a linked list node.
    /// Links and ends (front of the singly linked list) are updated by using the reference to the newly pushed node.
    ///
    /// ```rust ignore
    /// pub fn push_front(&mut self, value: T) {
    ///     self.col.mutate(value, |x, value| match x.ends().front() {
    ///         Some(prior_front) => {
    ///             let new_front = x.push_get_ref(value);
    ///             new_front.set_next(&x, prior_front);
    ///             x.set_ends(new_front);
    ///         }
    ///         None => {
    ///             let node = x.push_get_ref(value);
    ///             x.set_ends([Some(node), Some(node)]);
    ///         }
    ///     });
    /// }
    /// ```
    pub fn mutate<Move>(
        &mut self,
        value_to_move: Move,
        move_mutate_lambda: fn(SelfRefColMut<'_, 'a, V, T, P>, Move),
    ) {
        let vecmut = SelfRefColMut::new(self);
        move_mutate_lambda(vecmut, value_to_move);
    }

    /// Method allowing to mutate the collection and and return values from the collection.
    ///
    /// This method can only return types which implement `CanLeak`.
    /// Note that only `T` and `NodeIndex`, and types wrapping these two types, such as `Option` or `Vec`, implement `CanLeak`.
    /// This ensures the safety guarantees ara maintained.
    ///
    /// This method takes two arguments:
    /// * **`value_to_move`** is, as the name suggests, a value to be moved to the mutation lambda.
    /// Two common use cases to move this value to the lambda are:
    ///   * to add moved element(s) of `T` to the self referential collection,
    ///   * to use moved `NodeIndex` (indices) to access elements in constant time.
    /// * `mutate_get_lambda` is the expression defining the mutation.
    ///   * The lambda takes two parameters:
    ///     * the first parameter is the `SelfRefColMut` type which is the key for all `SelfRefNode` mutation and constant-time access methods;
    ///     * the second parameter is the value moved into the lambda, which is exactly the `value_to_move` parameter of this method.
    ///   * And it returns a type implementing `CanLeak` to make sure that safety guarantees of `SelfRefCol` are maintained.
    ///   * Note that the lambda is of a function pointer type; i.e., `fn`, rather than a function trait such as `FnOnce`.
    /// This is intentional and critical in terms of the safety guarantees.
    /// Its purpose is to prevent capturing data from the environment, as well as, prevent leaking vector references to outside of the lambda.
    ///
    /// This design allows to conveniently mutate the references within the vector without the complexity of lifetimes and borrow checker.
    /// Prior references can be broken, references can be rearranged or new references can be built easily.
    /// Furthermore, references by `NodeIndex` can be used for direct constant-time access to elements.
    /// This convenience is achieved by the encapsulation of all mutations within a non-capturing lambda.
    ///
    /// # Examples
    ///
    /// ## Example - Take out Value
    ///
    /// The following code block demonstrates the use of the `mutate_take` function to define the pop-front method of a singly linked list.
    /// Note that `self.vec` below is a `SelfRefCol`.
    /// Mutations are applied only if the vector is non-empty; i.e., there exists a **front**.
    /// When this is the case, the ends reference (front of the list) is updated.
    /// Furthermore, the prior-front's underlying data is taken out and returned from the lambda.
    /// This, in turn, is returned from the `pop_front` method, demonstrating safely removing elements from the self referential collection.
    ///
    /// ```rust ignore
    /// pub fn pop_front(&mut self) -> Option<T> {
    ///     self.col.mutate_take(|x| {
    ///         x.ends().front().map(|prior_front| {
    ///             let new_front = *prior_front.next().get();
    ///             let new_back = some_only_if(new_front.is_some(), x.ends().back());
    ///             x.set_ends([new_front, new_back]);
    ///
    ///             if let Some(new_front) = new_front {
    ///                 new_front.clear_prev(&x);
    ///             }
    ///             
    ///             prior_front.close_node_take_data(&x)
    ///         })
    ///     })
    /// }
    /// ```
    ///
    /// ## Example - Take out `NodeIndex`
    ///
    /// The following is exactly the same `push_front` example given in `mutate` example.
    /// However, we return an index, `NodeIndex`, to the pushed element this time.
    /// This index can later be used to have a constant time access to the element.
    ///
    /// ```rust ignore
    /// pub fn push_front(&mut self, value: T) -> NodeIndex<'_, V, T> {
    ///     self.col.mutate(value, |x, value| match x.ends().front() {
    ///         Some(prior_front) => {
    ///             let new_front = x.push_get_ref(value);
    ///             new_front.set_next(&x, prior_front);
    ///             x.set_ends(new_front);
    ///             new_front.index(&x)
    ///         }
    ///         None => {
    ///             let node = x.push_get_ref(value);
    ///             x.set_ends([Some(node), Some(node)]);
    ///             node.index(&x)
    ///         }
    ///     });
    /// }
    /// ```
    pub fn mutate_take<Move, Take: CanLeak<'a, V, T, P>>(
        &mut self,
        value_to_move: Move,
        move_mutate_take_lambda: fn(SelfRefColMut<'_, 'a, V, T, P>, Move) -> Take,
    ) -> Take {
        let vecmut = SelfRefColMut::new(self);
        move_mutate_take_lambda(vecmut, value_to_move)
    }

    /// This method takes three arguments:
    /// * `predicate` is the function to be used to select elements to be kept.
    /// * `collect` is the closure to collect the elements which does not satisfy the predicate and will be removed from this collection.
    /// * `mutate_filter_collect_lambda` is the expression defining the retain together with the mutation.
    ///   * In addition to `predicate` and `collect`, the lambda takes `SelfRefColMut` type which is the key for all `SelfRefNode` mutation methods to provide safety guarantees.
    ///   * Note that the lambda is of a function pointer type; i.e., `fn`, rather than a function trait such as `FnOnce`.
    /// This is intentional and critical in terms of the safety guarantees.
    /// Its purpose is to prevent capturing data from the environment, as well as, prevent leaking vector references to outside of the lambda.
    ///
    /// This method is a generalization of `mutate_take` which returns the element removed from the collection.
    /// In this method `collect(T)` is called on removed elements.
    /// This method might be doing nothing to drop the removed values, or might be pushing them to a captured collection such as a vector.
    /// Note that the signature of `Collect` is `FnMut(T)`; this makes sure that the function is called with removed/owned element values making sure that no references can leak out.
    ///
    /// This design allows to conveniently mutate the references within the vector without the complexity of lifetimes and borrow checker.
    /// Prior references can be broken, references can be rearranged or new references can be built easily and all in one function.
    /// This convenience while being safe is achieved by the encapsulation of all mutations within a non-capturing lambda.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_selfref_col::*;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// struct Var;
    /// impl<'a> Variant<'a, String> for Var {
    ///     type Storage = NodeDataLazyClose<String>;
    ///     type Prev = NodeRefSingle<'a, Self, String>;
    ///     type Next = NodeRefsVec<'a, Self, String>;
    ///     type Ends = NodeRefsArray<'a, 2, Self, String>;
    ///     type MemoryReclaim = MemoryReclaimNever;
    /// }
    ///
    /// // build up collection
    /// let mut col = SelfRefCol::<Var, _>::new();
    /// let values = ['a', 'b', 'c', 'd', 'e'];
    /// col.mutate(values.map(|x| x.to_string()), |x, vals| {
    ///     for value in vals {
    ///         let _ = x.push_get_ref(value);
    ///     }
    /// });
    ///
    /// let taboo_list = ['a', 's', 'd', 'f'];
    /// let taboo_list = taboo_list.map(|x| x.to_string());
    /// let is_allowed = |c: &String| !taboo_list.contains(c);
    ///
    /// let mut collected = vec![];
    /// let mut collect = |c| collected.push(c);
    ///
    /// col.mutate_filter_collect(&is_allowed, &mut collect, |x, predicate, collect| {
    ///     for i in 0..x.len() {
    ///         let node = x.get_node(i).expect("is-some");
    ///         if let Some(value) = node.data() {
    ///             if !predicate(value) {
    ///                 collect(node.close_node_take_data(&x));
    ///             }
    ///         }
    ///     }
    /// });
    ///
    /// assert_eq!(3, col.len());
    /// assert_eq!(&['a'.to_string(), 'd'.to_string()], collected.as_slice());
    /// ```
    pub fn mutate_filter_collect<Predicate, Collect>(
        &mut self,
        predicate: &Predicate,
        collect: &mut Collect,
        mutate_filter_collect_lambda: fn(SelfRefColMut<'_, 'a, V, T, P>, &Predicate, &mut Collect),
    ) where
        Predicate: Fn(&T) -> bool,
        Collect: FnMut(T),
    {
        let vecmut = SelfRefColMut::new(self);
        mutate_filter_collect_lambda(vecmut, predicate, collect);
    }

    // helpers
    pub(crate) fn memory_reclaimed(&mut self) {
        self.memory_reclaim_policy = self.memory_reclaim_policy.successor_state();
    }
}

type RecursiveSplitVec<'a, V, T> = SplitVec<Node<'a, V, T>, Recursive>;
type RecursiveSelfRefColMut<'rf, 'a, V, T> =
    SelfRefColMut<'rf, 'a, V, T, RecursiveSplitVec<'a, V, T>>;

impl<'a, V, T> SelfRefCol<'a, V, T, SplitVec<Node<'a, V, T>, Recursive>>
where
    V: Variant<'a, T>,
{
    /// This method appends another self collection to this collection.
    /// Note that this method is available in self referential collections using an underlying pinned vector with
    /// [`orx_split_vec::Recursive`](https://docs.rs/orx-split-vec/latest/orx_split_vec/struct.Recursive.html) growth.
    /// This allows appending underlying vectors in constant time.
    ///
    /// This method takes the following arguments:
    /// * `other` is the other self referential collection to be appended to this collection.
    /// * `value_to_move` is, as the name suggests, a value to be moved to the mutation lambda.
    /// * `append_mutate_lambda` is the expression defining the mutation.
    ///   * The lambda takes three parameters.
    ///   * The first parameter is the `SelfRefColMut` type which is the mutation key for this collection.
    ///   * The second parameter is the `SelfRefColMut` key of the `other` collection.
    ///   * The third parameter is the value moved into the lambda, which is exactly the `value_to_move` parameter of this method.
    ///   * Note that the lambda is of a function pointer type; i.e., `fn`, rather than a function trait such as `FnOnce`.
    /// This is intentional and critical in terms of the safety guarantees.
    /// Its purpose is to prevent capturing data from the environment, as well as, prevent leaking vector references to outside of the lambda.
    ///
    /// This design allows to conveniently mutate the references within the vector without the complexity of lifetimes and borrow checker.
    /// Prior references can be broken, references can be rearranged or new references can be built easily.
    /// Furthermore, references by `NodeIndex` can be used for direct constant-time access to elements.
    /// This convenience is achieved by the encapsulation of all mutations within a non-capturing lambda.
    ///
    /// # Example
    ///
    /// The following code block demonstrates the use of the `move_append_mutate` function to define the append-front method of a singly linked list.
    /// The method appends the `other` list to the front of the `self` list in ***O(1)*** time complexity.
    ///
    /// Note that appending the underlying storages are handled automatically by `SelfRefCol`.
    /// The lambda, taking mutation keys of both collections being merged, is responsible for fixing references.
    /// In this example, one next-relation is established and ends (front and back) of the list are updated.
    ///
    /// ```rust ignore
    /// pub fn append_front(&mut self, other: Self) {
    ///     self.col.move_append_mutate(other.col, (), |x, y, _| {
    ///         match (x.ends().front(), y.ends().back()) {
    ///             (Some(a), Some(b)) => {
    ///                 b.set_next(&x, a);
    ///                 x.set_ends([y.ends().front(), x.ends().back()]);
    ///             }
    ///             (None, Some(_)) => {
    ///                 x.set_ends([y.ends().front(), y.ends().back()]);
    ///             }
    ///             _ => {}
    ///         };
    ///         None
    ///     });
    /// }
    /// ```
    pub fn append_mutate<Move>(
        &mut self,
        other: Self,
        value_to_move: Move,
        append_mutate_lambda: fn(
            RecursiveSelfRefColMut<'_, 'a, V, T>,
            RecursiveSelfRefColMut<'_, 'a, V, T>,
            Move,
        ),
    ) {
        self.len += other.len;
        let mut_other = unsafe { into_mut(&other) };
        self.pinned_vec.append(other.pinned_vec);
        let x = SelfRefColMut::new(self);
        let y = SelfRefColMut::new(mut_other);
        append_mutate_lambda(x, y, value_to_move);
    }
}

impl<'a, V, T, P> Default for SelfRefCol<'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{
        MemoryReclaimNever, NodeData, NodeDataLazyClose, NodeRefSingle, NodeRefs, NodeRefsArray,
        NodeRefsVec,
    };
    use float_cmp::approx_eq;

    #[derive(Debug, Clone, Copy)]
    struct Var;
    impl<'a> Variant<'a, String> for Var {
        type Storage = NodeDataLazyClose<String>;
        type Prev = NodeRefSingle<'a, Self, String>;
        type Next = NodeRefsVec<'a, Self, String>;
        type Ends = NodeRefsArray<'a, 2, Self, String>;
        type MemoryReclaim = MemoryReclaimNever;
    }

    #[test]
    fn new_default() {
        let vec = SelfRefCol::<Var, _>::new();
        assert!(vec.pinned_vec.is_empty());
        assert!(vec.ends().get()[0].is_none());
        assert!(vec.ends().get()[1].is_none());

        let vec = SelfRefCol::<Var, _>::default();
        assert!(vec.pinned_vec.is_empty());
        assert!(vec.ends().get()[0].is_none());
        assert!(vec.ends().get()[1].is_none());
    }

    #[test]
    fn clear() {
        let mut col = SelfRefCol::<Var, _>::new();

        let values = ["a", "b", "c", "d"].map(|x| x.to_string());
        col.mutate(values, |x, values| {
            for val in values {
                let rf = x.push_get_ref(val);
                x.set_ends([Some(rf), Some(rf)]);
            }
        });

        assert_eq!(col.pinned_vec.len(), 4);

        col.clear();
        assert!(col.pinned_vec.is_empty());
        assert!(col.ends().get()[0].is_none());
        assert!(col.ends().get()[1].is_none());
        assert!(col.is_empty());
        assert_eq!(0, col.len());
    }

    #[test]
    fn mutate() {
        let mut vec = SelfRefCol::<Var, _>::new();

        let text = String::from("a");
        vec.mutate(text, |x, a| {
            let _ = x.push_get_ref(a);
        });
        assert_eq!(vec.pinned_vec.len(), 1);
        assert_eq!(vec.pinned_vec[0].data.get().unwrap(), "a");

        vec.mutate(String::from("b"), |x, b| {
            let _ = x.push_get_ref(b);
        });

        assert_eq!(vec.pinned_vec.len(), 2);
        assert_eq!(vec.pinned_vec[0].data.get().unwrap(), "a");
        assert_eq!(vec.pinned_vec[1].data.get().unwrap(), "b");
    }

    #[test]
    fn mutate_take() {
        let mut vec = SelfRefCol::<Var, _>::new();

        let text = String::from("a");
        vec.mutate(text.clone(), |x, a| {
            let ref_a = x.push_get_ref(a);
            x.set_ends([Some(ref_a), None]);
        });

        let text_back = vec.mutate_take((), |x, _| {
            let first = x.ends().get()[0];
            let data = first.map(|n| n.close_node_take_data(&x));
            x.set_ends([None, None]);
            data
        });

        assert_eq!(Some(text), text_back);
    }

    #[test]
    fn move_mutate_take() {
        let mut vec = SelfRefCol::<Var, _>::new();

        // when empty
        let taken: Option<String> = vec.mutate_take("a".to_string(), |x, a| {
            let _ = x.push_get_ref(a);
            None
        });
        assert!(taken.is_none());

        // with some taken value
        let taken = vec.mutate_take(["b".to_string(), "c".to_string()], |x, vals| match vals {
            [b, c] => {
                let ref_b = x.push_get_ref(b);
                Some(x.swap_data(ref_b, c))
            }
        });
        assert_eq!(taken, Some("b".to_string()));
    }

    #[test]
    fn mutate_filter_collect() {
        let taboo_list = ['a', 's', 'd', 'f'];
        let taboo_list = taboo_list.map(|x| x.to_string());
        let is_allowed = |c: &String| !taboo_list.contains(c);

        // when empty
        let mut col = SelfRefCol::<Var, _>::new();
        let mut vec = vec![];
        let mut collect = |c| vec.push(c);
        col.mutate_filter_collect(&is_allowed, &mut collect, |x, predicate, collect| {
            for i in 0..x.len() {
                let node = x.get_node(i).expect("is-some");
                if let Some(value) = node.data() {
                    if !predicate(value) {
                        collect(node.close_node_take_data(&x));
                    }
                }
            }
        });
        assert!(col.is_empty());
        assert!(vec.is_empty());

        // when single item
        let mut col = SelfRefCol::<Var, _>::new();
        col.mutate("a".to_string(), |x, a| {
            let _ = x.push_get_ref(a);
        });
        let mut vec = vec![];
        let mut collect = |c| vec.push(c);
        col.mutate_filter_collect(&is_allowed, &mut collect, |x, predicate, collect| {
            for i in 0..x.len() {
                let node = x.get_node(i).expect("is-some");
                if let Some(value) = node.data() {
                    if !predicate(value) {
                        collect(node.close_node_take_data(&x));
                    }
                }
            }
        });
        assert!(col.is_empty());
        assert_eq!(&['a'.to_string()], vec.as_slice());

        // when multiple items
        let mut col = SelfRefCol::<Var, _>::new();
        let values = ['a', 'b', 'c', 'd', 'e'];
        col.mutate(values.map(|x| x.to_string()), |x, vals| {
            for value in vals {
                let _ = x.push_get_ref(value);
            }
        });
        let mut vec = vec![];
        let mut collect = |c| vec.push(c);
        col.mutate_filter_collect(&is_allowed, &mut collect, |x, predicate, collect| {
            for i in 0..x.len() {
                let node = x.get_node(i).expect("is-some");
                if let Some(value) = node.data() {
                    if !predicate(value) {
                        collect(node.close_node_take_data(&x));
                    }
                }
            }
        });
        assert_eq!(3, col.len());
        assert_eq!(
            &['b'.to_string(), 'c'.to_string(), 'e'.to_string()],
            col.pinned_vec
                .iter()
                .filter_map(|x| x.data())
                .cloned()
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(&['a'.to_string(), 'd'.to_string()], vec.as_slice());
    }

    #[test]
    fn move_append_mutate() {
        let mut col = SelfRefCol::<Var, _>::new();
        col.mutate(["a", "b", "c"].map(|x| x.to_string()), |x, values| {
            for val in values {
                let _ = x.push_get_ref(val);
            }
            x.set_ends([x.first_node(), x.last_node()]);
        });

        let mut other = SelfRefCol::<Var, _>::new();
        other.mutate(["d", "e"].map(|x| x.to_string()), |x, values| {
            for val in values {
                let _ = x.push_get_ref(val);
            }
            x.set_ends([x.first_node(), x.last_node()]);
        });

        col.append_mutate(other, (), |x, y, _| {
            x.set_ends([x.first_node(), y.last_node()]);
        });

        assert_eq!(col.pinned_vec.len(), 5);
        assert_eq!(col.pinned_vec[0].data.get().unwrap(), "a");
        assert_eq!(col.pinned_vec[1].data.get().unwrap(), "b");
        assert_eq!(col.pinned_vec[2].data.get().unwrap(), "c");
        assert_eq!(col.pinned_vec[3].data.get().unwrap(), "d");
        assert_eq!(col.pinned_vec[4].data.get().unwrap(), "e");
        assert_eq!(
            col.ends().get()[0].map(|x| x.data().unwrap().as_str()),
            Some(&"a").copied()
        );
        assert_eq!(
            col.ends().get()[1].map(|x| x.data().unwrap().as_str()),
            Some(&"e").copied()
        );
    }

    #[test]
    fn reclaim_closed_nodes() {
        let mut col = SelfRefCol::<Var, _>::new();
        let values = ['a', 'b', 'c', 'd', 'e', 'f'].map(|x| x.to_string());
        let [a, b, c, _, _, _] = col.mutate_take(values, |x, values| {
            values.map(|val| x.push_get_ref(val).index(&x))
        });

        assert!(approx_eq!(f32, col.node_utilization(), 6.0 / 6.0, ulps = 2));

        col.mutate_take(a, |x, a| a.as_ref(&x).close_node_take_data(&x));
        assert!(approx_eq!(f32, col.node_utilization(), 5.0 / 6.0, ulps = 2));

        col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x));
        assert!(approx_eq!(f32, col.node_utilization(), 4.0 / 6.0, ulps = 2));

        col.mutate_take(c, |x, c| c.as_ref(&x).close_node_take_data(&x));
        assert!(approx_eq!(f32, col.node_utilization(), 3.0 / 6.0, ulps = 2));

        col.reclaim_closed_nodes();

        dbg!(col.node_utilization());
        assert!(approx_eq!(f32, col.node_utilization(), 3.0 / 3.0, ulps = 2));
    }
}
