use super::NodePtr;
use crate::{MemoryPolicy, MemoryState, Node, SelfRefCol, Variant};
use core::fmt::Debug;
use orx_pinned_vec::PinnedVec;

/// A node index providing safe and constant time access to elements
/// of the self referential collection.
pub struct NodeIdx<V: Variant> {
    ptr: *mut Node<V>,
    state: MemoryState,
}

impl<V: Variant> core::hash::Hash for NodeIdx<V> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
        self.state.hash(state);
    }
}

// Only the pointer is copied, so "V" does not need to be copy itself.
impl<V: Variant> Copy for NodeIdx<V> {}

impl<V: Variant> Clone for NodeIdx<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V: Variant> Debug for NodeIdx<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NodeIdx")
            .field("ptr", &self.ptr)
            .field("state", &self.state)
            .finish()
    }
}

impl<V: Variant> PartialEq for NodeIdx<V> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr && self.state == other.state
    }
}

impl<V: Variant> Eq for NodeIdx<V> {}

impl<V> NodeIdx<V>
where
    V: Variant,
{
    /// Creates a new index for the element at the given `node_ptr`
    /// and the collection with the given `state`.
    #[inline(always)]
    pub fn new(state: MemoryState, node_ptr: &NodePtr<V>) -> Self {
        Self {
            ptr: node_ptr.ptr_mut(),
            state,
        }
    }

    /// Checks whether or not the `state` of the index matches that of this index.
    #[inline(always)]
    pub fn is_in_state(&self, state: MemoryState) -> bool {
        self.state == state
    }

    #[inline(always)]
    pub(crate) fn ptr(&self) -> *const Node<V> {
        self.ptr
    }

    #[inline(always)]
    pub(crate) fn ptr_mut(&self) -> *mut Node<V> {
        self.ptr
    }

    /// Returns the node pointer if the index is in the same state as the `collection_state`,
    /// None otherwise.
    #[inline(always)]
    pub fn get_ptr(&self, collection_state: MemoryState) -> Option<*mut Node<V>> {
        self.state.eq(&collection_state).then_some(self.ptr)
    }

    /// Converts the node index into a node pointer.
    #[inline(always)]
    pub fn node_ptr(&self) -> NodePtr<V> {
        NodePtr::new(self.ptr)
    }

    /// Returns true only if this index is valid for the given `collection`.
    ///
    /// A node index is valid iff it satisfies the following two conditions:
    ///
    /// * It is created from the given `collection`.
    /// * Memory state of the `collection` has not changed since this index was created.
    #[inline(always)]
    pub fn is_valid_for<M, P>(&self, collection: &SelfRefCol<V, M, P>) -> bool
    where
        M: MemoryPolicy<V>,
        P: PinnedVec<Node<V>>,
    {
        self.state == collection.memory_state()
            && collection.nodes().contains_ptr(self.ptr)
            && unsafe { &*self.ptr }.is_active()
    }
}
