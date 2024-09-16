use crate::{
    node::Node, CoreCol, MemoryPolicy, MemoryState, NodeIdx, NodeIdxError, NodePtr, Variant,
};
use core::ops::{Deref, DerefMut};
use orx_pinned_vec::PinnedVec;

/// `SelfRefCol` is a core data structure to conveniently build safe and efficient self referential collections, such as linked lists and trees.
pub struct SelfRefCol<V, M, P>
where
    V: Variant,
    M: MemoryPolicy<V>,
    P: PinnedVec<Node<V>>,
{
    core: CoreCol<V, P>,
    policy: M,
    state: MemoryState,
}

impl<V, M, P> Default for SelfRefCol<V, M, P>
where
    V: Variant,
    M: MemoryPolicy<V>,
    P: PinnedVec<Node<V>> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V, M, P> Deref for SelfRefCol<V, M, P>
where
    V: Variant,
    M: MemoryPolicy<V>,
    P: PinnedVec<Node<V>>,
{
    type Target = CoreCol<V, P>;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl<V, M, P> DerefMut for SelfRefCol<V, M, P>
where
    V: Variant,
    M: MemoryPolicy<V>,
    P: PinnedVec<Node<V>>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl<V, M, P> SelfRefCol<V, M, P>
where
    V: Variant,
    M: MemoryPolicy<V>,
    P: PinnedVec<Node<V>>,
{
    /// Creates a new empty collection.
    pub fn new() -> Self
    where
        P: Default,
    {
        Self {
            core: CoreCol::new(),
            policy: M::default(),
            state: MemoryState::default(),
        }
    }

    /// Breaks the self referential collection into its core collection and memory state.
    pub fn into_inner(self) -> (CoreCol<V, P>, MemoryState) {
        let state = self.memory_state();
        (self.core, state)
    }

    pub(crate) fn from_raw_parts(core: CoreCol<V, P>, policy: M, state: MemoryState) -> Self {
        Self {
            core,
            policy,
            state,
        }
    }

    pub(crate) fn with_active_nodes(nodes: P) -> Self {
        Self {
            core: CoreCol::with_active_nodes(nodes),
            policy: M::default(),
            state: MemoryState::default(),
        }
    }

    // get

    /// Memory state of the collection.
    pub fn memory_state(&self) -> MemoryState {
        self.state
    }

    /// Memory policy of the collection.
    pub fn memory(&self) -> &M {
        &self.policy
    }

    /// Closes the node with the given `node_ptr`, returns its taken out value,
    /// and reclaims closed nodes if necessary.
    pub fn close_and_reclaim(&mut self, node_ptr: &NodePtr<V>) -> V::Item {
        let data = self.core.close(node_ptr);

        let state_changed = M::reclaim_closed_nodes(self, node_ptr);
        self.update_state(state_changed);

        data
    }

    /// If `state_changed` is true, proceeds to the next memory state.
    #[inline(always)]
    pub fn update_state(&mut self, state_changed: bool) {
        if state_changed {
            self.state = self.state.successor_state();
        }
    }

    /// Returns a reference to the node with the given `NodeIdx`;
    /// returns None if the index is invalid.
    #[inline(always)]
    pub fn node_from_idx(&self, idx: &NodeIdx<V>) -> Option<&Node<V>> {
        match idx.is_in_state(self.state) && self.nodes().contains_ptr(idx.ptr()) {
            true => Some(unsafe { &*idx.ptr() }),
            false => None,
        }
    }

    /// Tries to create a reference to the node with the given `NodeIdx`;
    /// returns the error if the index is invalid.
    #[inline(always)]
    pub fn try_node_from_idx(&self, idx: &NodeIdx<V>) -> Result<&Node<V>, NodeIdxError> {
        match self.nodes().contains_ptr(idx.ptr()) {
            true => match idx.is_in_state(self.state) {
                true => Ok(unsafe { &*idx.ptr() }),
                false => Err(NodeIdxError::ReorganizedCollection),
            },
            false => Err(NodeIdxError::OutOfBounds),
        }
    }

    /// Tries to get a valid pointer to the node with the given `NodeIdx`;
    /// returns the error if the index is invalid.
    #[inline(always)]
    pub fn try_get_ptr(&self, idx: &NodeIdx<V>) -> Result<NodePtr<V>, NodeIdxError> {
        match self.nodes().contains_ptr(idx.ptr()) {
            true => match idx.is_in_state(self.state) {
                true => {
                    let ptr = idx.ptr();
                    match unsafe { &*ptr }.is_active() {
                        true => Ok(NodePtr::new(ptr)),
                        false => Err(NodeIdxError::RemovedNode),
                    }
                }
                false => Err(NodeIdxError::ReorganizedCollection),
            },
            false => Err(NodeIdxError::OutOfBounds),
        }
    }

    // mut

    /// Clears the collection and changes the memory state.
    pub fn clear(&mut self) {
        self.core.clear_core();
        self.state = self.state.successor_state();
    }

    /// Returns a mutable reference to the node with the given `NodeIdx`;
    /// returns None if the index is invalid.
    #[inline(always)]
    pub fn node_mut_from_idx(&mut self, idx: &NodeIdx<V>) -> Option<&mut Node<V>> {
        match idx.is_in_state(self.state) && self.nodes().contains_ptr(idx.ptr()) {
            true => Some(unsafe { &mut *idx.ptr_mut() }),
            false => None,
        }
    }

    /// Tries to create a mutable reference to the node with the given `NodeIdx`;
    /// returns the error if the index is invalid.
    #[inline(always)]
    pub fn try_node_mut_from_idx(
        &mut self,
        idx: &NodeIdx<V>,
    ) -> Result<&mut Node<V>, NodeIdxError> {
        match self.nodes().contains_ptr(idx.ptr()) {
            true => match idx.is_in_state(self.state) {
                true => Ok(unsafe { &mut *idx.ptr_mut() }),
                false => Err(NodeIdxError::ReorganizedCollection),
            },
            false => Err(NodeIdxError::OutOfBounds),
        }
    }
}
