use super::{policy::MemoryPolicy, reclaimer::MemoryReclaimer};
use crate::{CoreCol, Node, NodePtr, Variant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;

/// Memory reclaim policy which triggers the reclaim operation whenever the node utilization
/// falls below a certain threshold.
///
/// Specifically, memory of closed nodes will be reclaimed whenever the ratio of closed nodes to all nodes exceeds one over `2^D`.
/// * when `D = 0`: memory will be reclaimed when utilization is below 0.00% (equivalent to never).
/// * when `D = 1`: memory will be reclaimed when utilization is below 50.00%.
/// * when `D = 2`: memory will be reclaimed when utilization is below 75.00%.
/// * when `D = 3`: memory will be reclaimed when utilization is below 87.50%.
/// * when `D = 4`: memory will be reclaimed when utilization is below 93.75%.
pub struct MemoryReclaimOnThreshold<const D: usize, V: Variant, R: MemoryReclaimer<V>> {
    phantom: PhantomData<(V, R)>,
}

impl<const D: usize, V: Variant, R: MemoryReclaimer<V>> Default
    for MemoryReclaimOnThreshold<D, V, R>
{
    fn default() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}

impl<const D: usize, V: Variant, R: MemoryReclaimer<V>> Clone
    for MemoryReclaimOnThreshold<D, V, R>
{
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl<const D: usize, V, R> MemoryPolicy<V> for MemoryReclaimOnThreshold<D, V, R>
where
    V: Variant,
    R: MemoryReclaimer<V>,
{
    fn reclaim_closed_nodes<P>(col: &mut CoreCol<V, P>, _closed_node_ptr: &NodePtr<V>) -> bool
    where
        P: PinnedVec<Node<V>>,
    {
        let num_active_nodes = col.len();
        let used = col.nodes().len();
        let allowed_vacant = used >> D;
        let num_vacant = used - num_active_nodes;

        match num_vacant <= allowed_vacant {
            true => false,
            false => {
                let nodes_moved = R::reclaim_nodes(col);
                col.nodes_mut().truncate(num_active_nodes);
                nodes_moved
            }
        }
    }
}
