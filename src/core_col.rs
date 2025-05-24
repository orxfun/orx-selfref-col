use crate::{NodePtr, Refs, Utilization, Variant, node::Node};
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Recursive, SplitVec};

/// Core collection of the self referential collection.
pub struct CoreCol<V, P>
where
    V: Variant,
    P: PinnedVec<Node<V>>,
{
    nodes: P,
    ends: V::Ends,
    len: usize,
}

impl<V, P> Default for CoreCol<V, P>
where
    V: Variant,
    P: PinnedVec<Node<V>> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V, P> CoreCol<V, P>
where
    V: Variant,
    P: PinnedVec<Node<V>>,
{
    /// Creates a new empty collection.
    pub fn new() -> Self
    where
        P: Default,
    {
        Self {
            nodes: P::default(),
            ends: Refs::empty(),
            len: 0,
        }
    }

    pub(crate) fn from_raw_parts(nodes: P, ends: V::Ends, len: usize) -> Self {
        Self { nodes, ends, len }
    }

    /// Destructs the collection into its inner pinned vec, ends and length.
    pub fn into_inner(self) -> (P, V::Ends, usize) {
        (self.nodes, self.ends, self.len)
    }

    pub(crate) fn with_active_nodes(nodes: P) -> Self {
        debug_assert!(nodes.iter().all(|x| x.data().is_some()));
        Self {
            len: nodes.len(),
            nodes,
            ends: Refs::empty(),
        }
    }

    // get

    /// Returns current node utilization of the collection.
    pub fn utilization(&self) -> Utilization {
        Utilization {
            capacity: self.nodes.capacity(),
            num_active_nodes: self.len,
            num_closed_nodes: self.nodes.len() - self.len,
        }
    }

    /// Returns length of the self referential collection.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether or not the self referential collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the underlying nodes storage.
    #[inline(always)]
    pub fn nodes(&self) -> &P {
        &self.nodes
    }

    /// Returns a reference to the node with the given `node_ptr`.
    #[inline(always)]
    pub fn node(&self, node_ptr: &NodePtr<V>) -> &Node<V> {
        unsafe { &*node_ptr.ptr_mut() }
    }

    /// Returns the position of the node with the given `node_ptr`,
    /// None if the pointer is not valid.
    #[inline(always)]
    pub fn position_of(&self, node_ptr: &NodePtr<V>) -> Option<usize> {
        self.nodes.index_of_ptr(node_ptr.ptr_mut())
    }

    /// Returns the position of the node with the given `node_ptr`.
    ///
    /// # Panics
    ///
    /// Panics if the pointer is not valid.
    #[inline(always)]
    pub fn position_of_unchecked(&self, node_ptr: &NodePtr<V>) -> usize {
        self.nodes
            .index_of_ptr(node_ptr.ptr_mut())
            .expect("Pointer does not belong to the collection")
    }

    /// Returns a reference to the data.
    ///
    /// # Panics
    ///
    /// Panics if the node is already closed.
    ///
    /// # Safety
    ///
    /// Does not perform bounds check; hence, the caller must guarantee that the
    /// `node_ptr` belongs to (created from) this collection.
    #[inline(always)]
    pub unsafe fn data_unchecked(&self, node_ptr: &NodePtr<V>) -> &V::Item {
        unsafe { &*node_ptr.ptr_mut() }
            .data()
            .expect("node is closed")
    }

    /// Returns a reference to the ends of the collection.
    #[inline(always)]
    pub fn ends(&self) -> &V::Ends {
        &self.ends
    }

    /// Returns the pointer of the element with the given `node_position`
    /// in the underlying nodes storage.
    ///
    /// # Panics
    ///
    /// Panics if the `node_position` is out of bounds.
    #[inline(always)]
    pub fn node_ptr_at_pos(&self, node_position: usize) -> NodePtr<V> {
        let ptr = self.nodes.get_ptr(node_position).expect("out-of-bounds");
        NodePtr::new(ptr as *mut Node<V>)
    }

    // mut

    pub(crate) fn clear_core(&mut self) {
        self.len = 0;
        self.ends.clear();
        self.nodes.clear();
    }

    /// Returns a mutable reference to the underlying nodes storage.
    #[inline(always)]
    pub fn nodes_mut(&mut self) -> &mut P {
        &mut self.nodes
    }

    /// Pushes the element with the given `data` and returns its pointer.
    pub fn push(&mut self, data: V::Item) -> NodePtr<V> {
        self.len += 1;
        let ptr = self.nodes.push_get_ptr(Node::new_free_node(data));
        NodePtr::new(ptr as *mut Node<V>)
    }

    /// Returns a mutable reference to the data.
    ///
    /// # Panics
    ///
    /// Panics if the node is already closed.
    ///
    /// # Safety
    ///
    /// Does not perform bounds check; hence, the caller must guarantee that the
    /// `node_ptr` belongs to (created from) this collection.
    #[inline(always)]
    pub unsafe fn data_mut_unchecked(&mut self, node_ptr: &NodePtr<V>) -> &mut V::Item {
        unsafe { &mut *node_ptr.ptr_mut() }
            .data_mut()
            .expect("node is closed")
    }

    /// Closes the node at the given `node_ptr` and returns its data.
    ///
    /// # Panics
    ///
    /// Panics if the node was already closed.
    #[inline(always)]
    pub fn close(&mut self, node_ptr: &NodePtr<V>) -> V::Item {
        self.len -= 1;
        unsafe { &mut *node_ptr.ptr_mut() }.close()
    }

    /// Closes the node at the given `node_ptr` and returns its data the node was active.
    /// Does nothing and returns None if the node was already closed.
    pub fn close_if_active(&mut self, node_ptr: &NodePtr<V>) -> Option<V::Item> {
        let node = unsafe { &mut *node_ptr.ptr_mut() };
        match node.is_active() {
            true => {
                self.len -= 1;
                Some(node.close())
            }
            false => None,
        }
    }

    /// Returns a mutable reference to the ends of the collection.
    pub fn ends_mut(&mut self) -> &mut V::Ends {
        &mut self.ends
    }

    /// Returns a mutable reference to the node with the given `node_ptr`.
    #[inline(always)]
    pub fn node_mut(&mut self, node_ptr: &NodePtr<V>) -> &mut Node<V> {
        unsafe { &mut *node_ptr.ptr_mut() }
    }

    /// Swaps the closed node at the `closed_position` with the active node
    /// at the `active_position`.
    pub fn move_node(&mut self, closed_position: usize, active_position: usize) {
        debug_assert!(closed_position < active_position);
        debug_assert!(self.nodes[closed_position].is_closed());
        debug_assert!(self.nodes[active_position].is_active());

        self.nodes_mut().swap(active_position, closed_position);
    }

    // data
    /// Swaps the underlying data of the element at the given `node_ptr` with the `new_value`,
    /// and returns the old value.
    ///
    /// # Panics
    ///
    /// Panics if the node was already closed.
    pub fn swap_data(&mut self, node_ptr: &NodePtr<V>, new_value: V::Item) -> V::Item {
        let node = unsafe { &mut *node_ptr.ptr_mut() };
        node.swap_data(new_value)
    }
}

impl<V> CoreCol<V, SplitVec<Node<V>, Recursive>>
where
    V: Variant,
{
    /// Appends the `nodes` to this collection.
    pub fn append_nodes(&mut self, nodes: SplitVec<Node<V>, Recursive>) {
        self.len += nodes.len();
        self.nodes.append(nodes)
    }
}
