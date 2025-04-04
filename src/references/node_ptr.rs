use crate::{Node, Variant};
use core::fmt::Debug;

/// A wrapper around a node pointer.
pub struct NodePtr<V: Variant> {
    ptr: *mut Node<V>,
}

impl<V: Variant> PartialEq for NodePtr<V> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<V: Variant> Debug for NodePtr<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NodeIdx")
            .field("ptr", &(self.ptr as usize))
            .finish()
    }
}

impl<V: Variant> Clone for NodePtr<V> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<V: Variant> NodePtr<V> {
    /// Creates a new node pointer by wrapping the given `ptr`.
    pub fn new(ptr: *const Node<V>) -> Self {
        Self {
            ptr: ptr as *mut Node<V>,
        }
    }

    /// Returns the const raw pointer.
    #[inline(always)]
    pub fn ptr(&self) -> *const Node<V> {
        self.ptr
    }

    /// Returns the mutable raw pointer.
    #[inline(always)]
    pub fn ptr_mut(&self) -> *mut Node<V> {
        self.ptr
    }

    // unsafe api
    /// Returns a reference to the node.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// * this pointer is created from a self referential collection,
    /// * the collection is still alive, and finally,
    /// * the memory state of the collection has not changed since the pointer was created.
    #[inline]
    pub unsafe fn node(&self) -> &Node<V> {
        unsafe { &*self.ptr }
    }

    /// Returns a mutable reference to the node.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// * this pointer is created from a self referential collection,
    /// * the collection is still alive, and finally,
    /// * the memory state of the collection has not changed since the pointer was created.
    #[inline]
    pub unsafe fn node_mut(&self) -> &mut Node<V> {
        unsafe { &mut *self.ptr }
    }
}
