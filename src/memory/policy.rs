use crate::{CoreCol, Node, NodePtr, Variant};
use orx_pinned_vec::PinnedVec;

/// Policy which determines how the memory of closed nodes will be reclaimed and made useful.
///
/// Two main implementors are:
/// * [`MemoryReclaimOnThreshold::<D>`] reclaims unused holes whenever the utilization of the memory falls below a constant threshold determined by `D`.
/// This could be considered as the flexible and general approach.
/// * [`MemoryReclaimNever`] which never reclaims the holes due to popped or removed; i.e., closed, nodes.
/// This approach has the advantage that a `NodeIndex` is never invalidated due to memory reorganization.
/// Note that it still allows to reclaim closed nodes manually.
/// Therefore, it fits very well to situations where
///   * removals from the list are not substantial, or
///   * having valid indices is crucial.
///
/// [`MemoryReclaimOnThreshold::<D>`]: crate::MemoryReclaimOnThreshold
/// [`MemoryReclaimNever`]: crate::MemoryReclaimNever
pub trait MemoryPolicy<V: Variant>: Clone + Default {
    /// Reclaims closed nodes.
    ///
    /// Assume that **A** below stands for active nodes and **x** designates a closed or popped node.
    /// If the underlying storage has the following layout at a certain stage:
    /// * `[ x, x, A, x, A, A, A, x, A, x ]`
    ///
    /// the reclaimer first reorganizes the nodes so that we have:
    /// * `[ A, A, A, A, A, x, x, x, x, x ]`
    ///
    /// and next trims the storage to reclaim memory
    /// * `[ A, A, A, A, A ]`
    ///
    /// Note that the order of the **A**s might change which is not relevant since this is self-referential-collection;
    /// i.e., the order is defined by the links among nodes.
    ///
    /// # Possible Strategies
    ///
    /// Memory reclaim policies define this operation:
    /// * [`MemoryReclaimNever`]: The manual policy which never automatically runs this operation,
    ///   gives the control completely to the user.
    ///   This fits best the use cases where we want to ensure the validity of stored node references or indices
    ///   as long as we do not change them.
    ///   * the user can never call `reclaim_closed_nodes` to ensure that the indices are always valid; or
    ///   * reclaims closed nodes whenever it is possible to refresh indices or prior indices are no longer required.
    /// * [`MemoryReclaimOnThreshold`]: Automatically reorganizes self whenever the utilization of memory falls
    ///   below a predefined threshold. This is the setting fitting most of the use cases.
    ///
    /// [`MemoryReclaimNever`]: crate::MemoryReclaimNever
    /// [`MemoryReclaimOnThreshold`]: crate::MemoryReclaimOnThreshold
    fn reclaim_closed_nodes<P>(col: &mut CoreCol<V, P>, closed_node_ptr: &NodePtr<V>) -> bool
    where
        P: PinnedVec<Node<V>>;
}
