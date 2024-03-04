use crate::{
    variants::memory_reclaim::MemoryReclaimPolicy, Node, SelfRefCol, SelfRefColMut, Variant,
};
use orx_split_vec::prelude::PinnedVec;

/// An index to a node in a `SelfRefCol` providing ***O(1)*** access time to the corresponding node.
///
/// It can be considered as a reference to a node which is safe to leak outside the collection.
/// * This is analogous to the `usize` index of an element of a standard vector which can be stored independently of the vector as a value.
/// * However, `NodeIndex` provides additional safety and correctness guarantees since validity of references is of utmost importance for self referential collections.
///
/// An index can be obtained from mutation methods of `SelfRefCol` which conventionally contain `take` in its name, such as `mutate_take` or `mutate_take`.
/// Furthermore, the index can be stored completely independently of the collection as a value.
///
/// However, when a node index is used to access the node of the collection, its correctness is guaranteed (see the Safety section for details).
/// When in doubt, `is_valid_for_collection` method can be used.
///
/// # Safety
///
/// * when `self.is_valid_for_collection(collection)` returns true; the node index can be converted into a valid node reference.
/// * when `self.is_valid_for_collection(collection)` returns false; the conversion to a node reference correctly returns None, this might lead to a panic depending on how the reference is used.
///
/// Invalid node index can be observed if at least one of the following three cases is observed.
/// 1. If the `NodeIndex` is created for another collection.
/// 2. If node which returned this `NodeIndex` on insertion is removed from the `collection`.
/// 3. If positions of the nodes were reorganized in order to optimize memory usage, which most likely invalidated this index.
///
/// ## 1. `NodeIndex` created for another collection
///
/// This requirement straightforward. Consider the corresponding demonstration with a standard vector, where index of an element is analogous to `NodeIndex` here.
///
/// ```rust
/// let mut vec = vec![];
///
/// vec.push('a');
/// vec.push('b');
/// let index_x = {
///     vec.push('x');
///     vec.len() - 1
/// };
/// vec.push('c');
///
/// // Case 1
/// let node_x = vec.get(index_x);
/// assert_eq!(node_x, Some(&'x')); // ✓ index used on the correct collection and returned the correct value
///
/// // Case 2
/// let vec2 = vec!['0', '1', '2', '3'];
/// let node_x = vec2.get(index_x);
/// assert_eq!(node_x, Some(&'2')); // 🗙 index used on wrong collection, and obtained a wrong value
///
/// // Case 3
/// let vec3 = vec!['.'];
/// let node_x = vec3.get(index_x);
/// assert!(node_x.is_none()); // 🗙 index used on wrong collection, obtained None, although for a different reason
/// ```
///
/// While using `NodeIndex` in a `SelfRefCol`:
/// * in Case 1: we receive a valid reference to the correct node,
/// * in Case 2: we receive `None`, unlike the behavior above for vec,
/// * in Case 3: we receive `None` again.
///
/// Note that the way Case 2 is handled provides additional safety.
/// This safety is achieved with additional memory and computational cost compared to usize & Vec example.
/// However, this makes sense considering that the primary feature of self referential collections is
/// efficiency in following the references rather than accessing by indices.
///
/// ## 2. Node for which `NodeIndex` is removed from the collection
///
/// This requirement as well is straightforward, and below is the demonstration using the standard vector & index analogy.
///
/// ```rust
/// let mut vec = vec![];
///
/// vec.push('a');
/// vec.push('b');
/// let index_x = {
///     vec.push('x');
///     vec.len() - 1
/// };
/// vec.push('c');
///
/// // Case 4
/// vec.remove(2); // at this point, vec = ['a', 'b', 'c']
/// let node_x = vec.get(index_x);
/// assert_eq!(node_x, Some(&'c')); // 🗙 index used after node 'x' is removed from the collection, and grabbed a wrong value
/// ```
///
/// While using `NodeIndex` to access a `SelfRefCol` in Case 4, we would again receive None.
/// Similar to above, this is an additional correctness check.
///
/// ## 3. If node positions of the collection are reorganized
///
/// Note that such node reorganization is observed when memory reclaim policy of the collection is `MemoryReclaimOnThreshold`.
/// It will never be observed when `MemoryReclaimNever` is used.
/// This leads to the following rule of thumb:
/// * If amount of removals from our self referential collection is not high compared to the size of the collection,
/// it is beneficial to use `MemoryReclaimNever`. This has two advantages:
///   * Our collection will never spend time for reorganization.
/// However, the benefit here can be considered to be minor since memory checks are triggered only on removals which are rare in this case.
///   * More importantly, this third invalid node index case can never be observed. This leads to the following advantages:
///     * `NodeIndex` is as small as a pointer or `usize`. This is the minimum index size that can be achieved.
///     * Accessing through the `NodeIndex` skips this third validity check, which precisely an equality check of two 128-bit values.
/// * For all other cases where our collection continuously grows and shrinks, or when we do not require access by `NodeIndex`,
/// it is beneficial to use `MemoryReclaimOnThreshold`.
///
/// See below for a demonstration of the analogous situation in vector & index case.
///
/// ```rust
/// let mut vec = vec![];
///
/// vec.push('a');
/// vec.push('b');
/// let index_x = {
///     vec.push('x');
///     vec.len() - 1
/// };
/// vec.push('c');
///
/// // Case 5
/// vec.remove(0); // at this point, vec = ['b', 'x', 'c']
/// let node_x = vec.get(index_x);
/// assert_eq!(node_x, Some(&'c')); // 🗙 index used after nodes are reorganized changing location of 'x', and we grabbed a wrong value
/// ```
///
/// As above, when using a `NodeIndex` on a `SelfRefCol`, we would receive `None` for Case 5, rather than getting a reference to a wrong node.
#[derive(Copy)]
pub struct NodeIndex<'a, V, T>
where
    V: Variant<'a, T>,
{
    collection_key: V::MemoryReclaim,
    pub(crate) node_key: &'a Node<'a, V, T>,
}

impl<'a, V, T> Clone for NodeIndex<'a, V, T>
where
    V: Variant<'a, T>,
{
    fn clone(&self) -> Self {
        Self {
            collection_key: self.collection_key,
            node_key: self.node_key,
        }
    }
}

impl<'a, V, T> PartialEq for NodeIndex<'a, V, T>
where
    V: Variant<'a, T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.collection_key
            .is_same_collection_as(&other.collection_key)
            && self.node_key.ref_eq(other.node_key)
    }
}
impl<'a, V, T> Eq for NodeIndex<'a, V, T> where V: Variant<'a, T> {}

impl<'a, V, T> NodeIndex<'a, V, T>
where
    V: Variant<'a, T>,
{
    /// Creates a new node index with the given `collection_key` and `node_key`.
    ///
    /// * `node_key` is always a pointer size, a reference to the node at the point when the index is created;
    /// * `collection_key` is:
    ///   * zero-sized when the collection uses `MemoryReclaimNever` as its `MemoryReclaimPolicy` because in this case:
    ///     * a wrong collection case is caught by pinned collection's `contains_reference` test,
    ///     * a removed node case is caught by checking whether or not the node is active, and finally
    ///     * a reorganized memory positions case can never be observed.
    ///   * 128-bit sized when the collection uses `MemoryReclaimOnThreshold` as its `MemoryReclaimPolicy`:
    ///     * the first and second cases caught identically; however,
    ///     * the third case is caught by a uuid comparison.
    pub(crate) fn new(collection_key: V::MemoryReclaim, node_key: &'a Node<'a, V, T>) -> Self {
        Self {
            collection_key,
            node_key,
        }
    }

    /// Returns true only if all of of the following safety and correctness conditions hold:
    /// * this index is created from the given `collection`,
    /// * the node this index is created for still belongs to the `collection`; i.e., is not removed,
    /// * the node positions in the `collection` are not reorganized to reclaim memory.
    ///
    /// This conditions are sufficient to prove that the node index is valid and safe to use and will access the correct node.
    pub fn is_valid_for_collection<P>(&self, collection: &SelfRefCol<'a, V, T, P>) -> bool
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        self.collection_key
            .is_same_collection_as(&collection.memory_reclaim_policy)
            && collection.pinned_vec.contains_reference(self.node_key)
            && self.node_key.is_active()
    }

    /// Converts the node index to a reference to the node in the `collection`; returns None if `self.is_valid_for_collection(collection)` is false.
    ///
    /// `is_valid_for_collection(collection)` returns true if all of of the following safety and correctness conditions hold:
    /// * this index is created from the given `collection`,
    /// * the node this index is created for still belongs to the `collection`; i.e., is not removed,
    /// * the node positions in the `collection` are not reorganized to reclaim memory.
    #[inline(always)]
    pub fn get_ref<'rf, P>(
        &self,
        collection: &SelfRefColMut<'rf, 'a, V, T, P>,
    ) -> Option<&'a Node<'a, V, T>>
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        collection.index_to_maybe_ref(self)
    }

    /// Converts the node index to a reference to the node in the `collection`.
    /// The call panics if `self.is_valid_for_collection(collection)` is false; i.e., if this node index is not valid for the given `collection`.
    ///
    /// # Panics
    ///
    /// Panics if `is_valid_for_collection(collection)` returns false.
    ///
    /// Note that `is_valid_for_collection` returns true if all of of the following safety and correctness conditions hold:
    /// * this index is created from the given `collection`,
    /// * the node this index is created for still belongs to the `collection`; i.e., is not removed,
    /// * the node positions in the `collection` are not reorganized to reclaim memory.
    #[inline(always)]
    pub fn as_ref<'rf, P>(&self, collection: &SelfRefColMut<'rf, 'a, V, T, P>) -> &'a Node<'a, V, T>
    where
        P: PinnedVec<Node<'a, V, T>>,
    {
        collection.index_to_ref(self)
    }

    /// Converts the node index to a reference to the node in the `collection`.
    ///
    /// # Safety
    ///
    /// The safe conversion can be performed by `as_ref`, and a safer conversion by `get_ref`.
    ///
    /// This method is unsafe as the reference to the node held internally might have been invalidated between time it was created and it is used.
    ///
    /// Therefore, the caller takes responsibility that the references are not invalidated.
    /// In this case, omitting safety and correctness checks, this method provides the fastest access to the node.
    ///
    /// Let's call the collection that will be accessed by this index ``collection``.
    /// The caller can be confident that the node index is still valid for this `collection`
    /// if all of of the following safety and correctness conditions hold:
    /// * this index is created from the given `collection`,
    /// * the node this index is created for still belongs to the `collection`; i.e., is not removed,
    /// * the node positions in the `collection` are not reorganized to reclaim memory.
    #[inline(always)]
    pub unsafe fn as_ref_unchecked(&self) -> &'a Node<'a, V, T> {
        self.node_key
    }
}

#[cfg(test)]
mod tests {
    use orx_split_vec::{Recursive, SplitVec};

    use crate::{
        variants::memory_reclaim::MemoryReclaimPolicy, MemoryReclaimOnThreshold, Node,
        NodeDataLazyClose, NodeIndex, NodeRefSingle, NodeRefsArray, NodeRefsVec, SelfRefCol,
        SelfRefColMut, Variant,
    };

    #[derive(Clone, Copy, Debug)]
    struct Var;
    impl<'a> Variant<'a, char> for Var {
        type Storage = NodeDataLazyClose<char>;
        type Prev = NodeRefSingle<'a, Self, char>;
        type Next = NodeRefsVec<'a, Self, char>;
        type Ends = NodeRefsArray<'a, 2, Self, char>;
        type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    }

    #[test]
    fn new_clone() {
        let node = Node::<Var, _>::new_free_node('x');
        let reclaim: MemoryReclaimOnThreshold<2> = MemoryReclaimOnThreshold::default();

        let index = NodeIndex::new(reclaim, &node);
        #[allow(clippy::clone_on_copy)]
        let clone = index.clone();

        assert!(index.node_key.ref_eq(&node));
        assert!(clone.node_key.ref_eq(&node));

        assert!(<MemoryReclaimOnThreshold<2> as MemoryReclaimPolicy<
            '_,
            Var,
            _,
            _,
            _,
        >>::is_same_collection_as(
            &index.collection_key, &reclaim,
        ));

        assert!(<MemoryReclaimOnThreshold<2> as MemoryReclaimPolicy<
            '_,
            Var,
            _,
            _,
            _,
        >>::is_same_collection_as(
            &clone.collection_key, &reclaim,
        ));
    }

    #[test]
    fn eq() {
        let mut col = SelfRefCol::<Var, _>::new();

        let a1 = col.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));
        let a2 = col.mutate_take((), |x, _| x.first_node().expect("is-some").index(&x));

        assert!(a1.eq(&a2));
        assert_eq!(&a1, &a2);
    }

    #[test]
    fn is_valid_for_collection() {
        let mut col = SelfRefCol::<Var, _>::new();

        let a = col.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));
        assert!(a.is_valid_for_collection(&col));

        let b = col.mutate_take('b', |x, a| x.push_get_ref(a).index(&x));
        assert!(a.is_valid_for_collection(&col));
        assert!(b.is_valid_for_collection(&col));
    }

    #[test]
    fn is_invalid_belongs_to_different_collection() {
        let mut col1 = SelfRefCol::<Var, _>::new();
        let a = col1.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));

        let col2 = SelfRefCol::<Var, _>::new();

        assert!(!a.is_valid_for_collection(&col2));
    }

    #[test]
    fn is_invalid_because_removed() {
        let mut col = SelfRefCol::<Var, _>::new();
        let [a, b, c, d, e, f, g] = col
            .mutate_take(['a', 'b', 'c', 'd', 'e', 'f', 'g'], |x, values| {
                values.map(|val| x.push_get_ref(val).index(&x))
            });

        let removed_b = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // does not trigger reclaim yet
        assert_eq!(removed_b, 'b');

        assert!(a.is_valid_for_collection(&col));
        assert!(c.is_valid_for_collection(&col));
        assert!(d.is_valid_for_collection(&col));
        assert!(e.is_valid_for_collection(&col));
        assert!(f.is_valid_for_collection(&col));
        assert!(g.is_valid_for_collection(&col));

        assert!(!b.is_valid_for_collection(&col));
    }

    #[test]
    fn is_invalid_because_reorganized() {
        let mut col = SelfRefCol::<Var, _>::new();
        let [a, b, c] = col.mutate_take(['a', 'b', 'c'], |x, values| {
            values.map(|val| x.push_get_ref(val).index(&x))
        });

        let removed_b = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // triggers reclaim
        assert_eq!(removed_b, 'b');

        assert!(!a.is_valid_for_collection(&col));
        assert!(!b.is_valid_for_collection(&col));
        assert!(!c.is_valid_for_collection(&col));
    }

    #[test]
    fn get_as_ref_when_valid() {
        let mut col = SelfRefCol::<Var, _>::new();

        let a = col.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));
        let b = col.mutate_take('b', |x, a| x.push_get_ref(a).index(&x));

        col.mutate((a, b), |x, (a, b)| {
            assert_node_ref_is_valid(&x, a, 'a');
            assert_node_ref_is_valid(&x, b, 'b');
        });
    }

    #[test]
    fn get_ref_invalid_belongs_to_different_collection() {
        let mut col1 = SelfRefCol::<Var, _>::new();
        let a = col1.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));

        let mut col2 = SelfRefCol::<Var, _>::new();
        col2.mutate(a, |x, a| assert_node_ref_is_invalid(&x, a));
    }

    #[test]
    #[should_panic]
    fn as_ref_invalid_belongs_to_different_collection() {
        let mut col1 = SelfRefCol::<Var, _>::new();
        let a = col1.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));

        let mut col2 = SelfRefCol::<Var, _>::new();

        col2.mutate(a, |x, a| {
            a.as_ref(&x);
        });
    }

    #[test]
    fn get_ref_invalid_because_removed() {
        let mut col = SelfRefCol::<Var, _>::new();
        let [a, b, c, d, e, f, g] = col
            .mutate_take(['a', 'b', 'c', 'd', 'e', 'f', 'g'], |x, values| {
                values.map(|val| x.push_get_ref(val).index(&x))
            });

        let removed_b = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // does not trigger reclaim yet
        assert_eq!(removed_b, 'b');

        col.mutate((a, b, c, d, e, f, g), |x, (a, b, c, d, e, f, g)| {
            assert_node_ref_is_valid(&x, a, 'a');
            assert_node_ref_is_invalid(&x, b);
            assert_node_ref_is_valid(&x, c, 'c');
            assert_node_ref_is_valid(&x, d, 'd');
            assert_node_ref_is_valid(&x, e, 'e');
            assert_node_ref_is_valid(&x, f, 'f');
            assert_node_ref_is_valid(&x, g, 'g');
        });
    }

    #[test]
    #[should_panic]
    fn as_ref_invalid_because_removed() {
        let mut col = SelfRefCol::<Var, _>::new();
        let [_, b, _, _, _, _, _] = col
            .mutate_take(['a', 'b', 'c', 'd', 'e', 'f', 'g'], |x, values| {
                values.map(|val| x.push_get_ref(val).index(&x))
            });

        let removed_b = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // does not trigger reclaim yet
        assert_eq!(removed_b, 'b');

        col.mutate(b, |x, b| {
            b.as_ref(&x);
        });
    }

    #[test]
    fn get_ref_invalid_because_reorganized() {
        let mut col = SelfRefCol::<Var, _>::new();
        let [a, b, c] = col.mutate_take(['a', 'b', 'c'], |x, values| {
            values.map(|val| x.push_get_ref(val).index(&x))
        });

        let _ = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // triggers reclaim

        col.mutate((a, b, c), |x, (a, b, c)| {
            assert_node_ref_is_invalid(&x, a);
            assert_node_ref_is_invalid(&x, b);
            assert_node_ref_is_invalid(&x, c);
        });
    }

    #[test]
    #[should_panic]
    fn as_ref_invalid_because_reorganized() {
        let mut col = SelfRefCol::<Var, _>::new();
        let [a, b, _] = col.mutate_take(['a', 'b', 'c'], |x, values| {
            values.map(|val| x.push_get_ref(val).index(&x))
        });

        let _ = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // triggers reclaim

        col.mutate(a, |x, a| {
            a.as_ref(&x);
        });
    }

    // helpers
    fn assert_node_ref_is_valid<'a>(
        x: &SelfRefColMut<'_, 'a, Var, char, SplitVec<Node<'a, Var, char>, Recursive>>,
        node_index: NodeIndex<'a, Var, char>,
        expected_value: char,
    ) {
        assert_eq!(
            node_index.get_ref(x).and_then(|a| a.data()),
            Some(&expected_value)
        );
        assert_eq!(node_index.as_ref(x).data(), Some(&expected_value));
        assert_eq!(
            unsafe { node_index.as_ref_unchecked().data() },
            Some(&expected_value)
        );
    }

    fn assert_node_ref_is_invalid<'a>(
        x: &SelfRefColMut<'_, 'a, Var, char, SplitVec<Node<'a, Var, char>, Recursive>>,
        node_index: NodeIndex<'a, Var, char>,
    ) {
        assert!(node_index.get_ref(x).is_none());
    }
}
