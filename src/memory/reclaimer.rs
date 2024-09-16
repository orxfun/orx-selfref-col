use crate::{CoreCol, Node, Variant};
use orx_pinned_vec::PinnedVec;

/// Memory reclaimer which reorganizes the collection nodes and brings node utilization to 100%.
pub trait MemoryReclaimer<V>: Clone + Default
where
    V: Variant,
{
    /// Memory reclaimer which reorganizes the collection nodes and brings node utilization to 100%.
    fn reclaim_nodes<P>(col: &mut CoreCol<V, P>) -> bool
    where
        P: PinnedVec<Node<V>>;
}
