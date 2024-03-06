use crate::{
    nodes::index::NodeIndex, selfref_col_mut::into_ref, Node, NodeIndexError, SelfRefCol,
    SelfRefColMut, Variant,
};
use orx_split_vec::prelude::PinnedVec;
use std::ops::Deref;

/// Struct allowing to safely, conveniently and efficiently visit nodes of a self referential collection and take `CanLeak` values out.
///
/// # Safety
///
/// This struct holds an immutable reference to the underlying self referential collection.
///
/// This struct cannot be created externally.
/// It is only constructed by `SelfRefCol`s methods such as `visit` and `visit_take` methods.
/// You may find corresponding safety guarantees in the documentation of these methods, which in brief, relies on careful encapsulation
/// preventing any reference to leak into the collection or any node reference to leak out.
pub struct SelfRefColVisit<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
    'a: 'rf,
{
    pub(crate) col: &'rf SelfRefCol<'a, V, T, P>,
}

impl<'rf, 'a, V, T, P> SelfRefColVisit<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    pub(crate) fn new(vec: &'rf SelfRefCol<'a, V, T, P>) -> Self {
        Self { col: vec }
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
}

impl<'rf, 'a, V, T, P> Deref for SelfRefColVisit<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    type Target = SelfRefCol<'a, V, T, P>;

    fn deref(&self) -> &Self::Target {
        self.col
    }
}

impl<'rf, 'a, V, T, P> From<&SelfRefColMut<'rf, 'a, V, T, P>> for SelfRefColVisit<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    fn from(value: &SelfRefColMut<'rf, 'a, V, T, P>) -> Self {
        Self::new(unsafe { into_ref(value.col) })
    }
}
