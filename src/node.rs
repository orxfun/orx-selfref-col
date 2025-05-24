use crate::{Refs, Variant};
use core::fmt::Debug;

/// Node of the self referential collection.
pub struct Node<V>
where
    V: Variant,
{
    data: Option<V::Item>,
    prev: V::Prev,
    next: V::Next,
}

unsafe impl<V: Variant> Send for Node<V> where V::Item: Send {}

unsafe impl<V: Variant> Sync for Node<V> where V::Item: Sync {}

impl<V> Node<V>
where
    V: Variant,
{
    /// Creates a new active node with the given `data`, and `prev` and `next` references.
    pub fn new_active(data: V::Item, prev: V::Prev, next: V::Next) -> Self {
        Self {
            data: Some(data),
            prev,
            next,
        }
    }

    /// Creates a new active node with the given `data` but with no connections.
    pub fn new_free_node(data: V::Item) -> Self {
        Self {
            data: Some(data),
            prev: Refs::empty(),
            next: Refs::empty(),
        }
    }

    // consuming

    /// Takes and returns the data of the node, transitions the node into the closed state.
    pub fn into_data(self) -> Option<V::Item> {
        self.data
    }

    // ref

    /// Returns a reference to the data of the node; None if the node is already closed.
    pub fn data(&self) -> Option<&V::Item> {
        self.data.as_ref()
    }

    /// Returns a reference to the previous references.
    pub fn prev(&self) -> &V::Prev {
        &self.prev
    }

    /// Returns a reference to the next references.
    pub fn next(&self) -> &V::Next {
        &self.next
    }

    /// Returns true if the node is active, false if it is closed.
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.data.is_some()
    }

    /// Returns true if the node is closed, false if it is active.
    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        self.data.is_none()
    }

    // mut

    /// Returns a mutable reference to the underlying data.
    pub fn data_mut(&mut self) -> Option<&mut V::Item> {
        self.data.as_mut()
    }

    /// Returns a mutable reference to the previous references.
    pub fn prev_mut(&mut self) -> &mut V::Prev {
        &mut self.prev
    }

    /// Returns a mutable reference to the next references.
    pub fn next_mut(&mut self) -> &mut V::Next {
        &mut self.next
    }

    /// Closes the node and returns its data, and clears its connections.
    ///
    /// # Panics
    ///
    /// Panics if the node was already closed.
    pub fn close(&mut self) -> V::Item {
        self.prev.clear();
        self.next.clear();
        self.data.take().expect("must be an open node")
    }

    /// Swaps the data of the node with the `new_value` and returns the old value.
    ///
    /// # Panics
    ///
    /// Panics if the node was already closed.
    pub fn swap_data(&mut self, new_value: V::Item) -> V::Item {
        debug_assert!(self.is_active());
        self.data.replace(new_value).expect("must be active")
    }

    /// Closes the node and returns its data.
    ///
    /// Returns None if the node was already closed.
    pub fn take_data(&mut self) -> Option<V::Item> {
        self.data.take()
    }
}

impl<V: Variant> Debug for Node<V>
where
    V::Item: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Node")
            .field("data", &self.data)
            .field("prev", &self.prev)
            .field("next", &self.next)
            .finish()
    }
}
