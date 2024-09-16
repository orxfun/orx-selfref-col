use super::policy::MemoryPolicy;
use crate::{CoreCol, Node, NodePtr, Variant};
use orx_pinned_vec::PinnedVec;

/// A do-nothing `MemoryReclaimPolicy` which would never reclaim the memory of the closed nodes, leaving them as holes in the underlying storage.
///
/// This approach has the advantage that a `NodeIndex` is never invalidated due to an automatic memory reorganization.
///
/// Furthermore, node utilization can still be maximized by manually calling `reclaim_closed_nodes` method.
#[derive(Default, Clone, Copy)]
pub struct MemoryReclaimNever;

impl<V: Variant> MemoryPolicy<V> for MemoryReclaimNever {
    #[inline(always)]
    fn reclaim_closed_nodes<P>(_col: &mut CoreCol<V, P>, _closed_node_ptr: &NodePtr<V>) -> bool
    where
        P: PinnedVec<Node<V>>,
    {
        false
    }
}
