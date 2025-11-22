use crate::{MemoryPolicy, Node, SelfRefCol, Variant};
use core::fmt::Debug;
use orx_pinned_vec::PinnedVec;

/// A wrapper around a node pointer.
pub struct NodePtr<V: Variant> {
    ptr: *mut Node<V>,
}

unsafe impl<V: Variant> Send for NodePtr<V> where V::Item: Send {}

unsafe impl<V: Variant> Sync for NodePtr<V> where V::Item: Sync {}

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

    /// Returns true if:
    ///
    /// * `collection` owns this `NodePtr`, and
    /// * the node, or corresponding element of the `collection`, that this pointer
    ///   is pointing at is still active;
    ///
    /// false otherwise.
    ///
    /// It is safe to use the unsafe methods of `NodePtr` if `is_valid_for(col)`
    /// returns true where `col` is the collection that the pointer is created from.
    #[inline(always)]
    pub fn is_valid_for<M, P>(&self, collection: &SelfRefCol<V, M, P>) -> bool
    where
        M: MemoryPolicy<V>,
        P: PinnedVec<Node<V>>,
    {
        collection.nodes().contains_ptr(self.ptr) && unsafe { &*self.ptr }.is_active()
    }

    /// Returns the const raw pointer to the node.
    ///
    /// # SAFETY
    ///
    /// This method is unsafe as `NodePtr` implements `Send` and `Sync`.
    ///
    /// It is safe dereference the received pointer when we can validate that the collection
    /// owning the node is alive with the same memory state when the node pointer was created.
    #[inline(always)]
    pub unsafe fn ptr(&self) -> *const Node<V> {
        self.ptr
    }

    /// Returns the mutable raw pointer to the node.
    ///
    /// # SAFETY
    ///
    /// This method is unsafe as `NodePtr` implements `Send` and `Sync`.
    ///
    /// It is safe dereference the received pointer when we can validate that the collection
    /// owning the node is alive with the same memory state when the node pointer was created.
    #[inline(always)]
    pub unsafe fn ptr_mut(&self) -> *mut Node<V> {
        self.ptr
    }

    /// Returns a reference to the node.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
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
    ///
    /// * this pointer is created from a self referential collection,
    /// * the collection is still alive, and finally,
    /// * the memory state of the collection has not changed since the pointer was created.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn node_mut(&self) -> &mut Node<V> {
        unsafe { &mut *self.ptr }
    }
}
