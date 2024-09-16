use crate::{
    MemoryReclaimNever, MemoryReclaimOnThreshold, MemoryReclaimer, Node, SelfRefCol, Variant,
};
use orx_pinned_vec::PinnedVec;

impl<const D: usize, R, V, P> From<SelfRefCol<V, MemoryReclaimNever, P>>
    for SelfRefCol<V, MemoryReclaimOnThreshold<D, V, R>, P>
where
    V: Variant,
    R: MemoryReclaimer<V>,
    P: PinnedVec<Node<V>>,
{
    fn from(value: SelfRefCol<V, MemoryReclaimNever, P>) -> Self {
        let (core, state) = value.into_inner();
        Self::from_raw_parts(core, Default::default(), state)
    }
}

impl<const D: usize, R, V, P> From<SelfRefCol<V, MemoryReclaimOnThreshold<D, V, R>, P>>
    for SelfRefCol<V, MemoryReclaimNever, P>
where
    V: Variant,
    R: MemoryReclaimer<V>,
    P: PinnedVec<Node<V>>,
{
    fn from(value: SelfRefCol<V, MemoryReclaimOnThreshold<D, V, R>, P>) -> Self {
        let (core, state) = value.into_inner();
        Self::from_raw_parts(core, Default::default(), state)
    }
}
