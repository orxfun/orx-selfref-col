use crate::{
    memory_reclaim::memory_state::MemoryState, Node, NodeDataLazyClose, NodeRefNone, NodeRefSingle,
    NodeRefs, NodeRefsArray, NodeRefsVec, SelfRefColMut, Variant,
};
use orx_split_vec::prelude::PinnedVec;

pub trait MemoryReclaimPolicy<'a, V, T, Prev, Next>: Default + Clone + Copy
where
    V: Variant<'a, T, Prev = Prev, Next = Next>,
    Prev: NodeRefs<'a, V, T>,
    Next: NodeRefs<'a, V, T>,
{
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a;

    fn is_same_collection_as(&self, collection: &Self) -> bool
    where
        Self: MemoryReclaimPolicy<'a, V, T, Prev, Next>;

    fn successor_state(&self) -> Self;
}

// never
/// A do-nothing `MemoryReclaimPolicy` which would never reclaim the memory of the closed nodes, leaving them as holes in the underlying storage.
#[derive(Default, Clone, Copy)]
pub struct MemoryReclaimNever;

impl<'a, V, T, Prev, Next> MemoryReclaimPolicy<'a, V, T, Prev, Next> for MemoryReclaimNever
where
    V: Variant<'a, T, Prev = Prev, Next = Next>,
    Prev: NodeRefs<'a, V, T>,
    Next: NodeRefs<'a, V, T>,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(_vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
    }

    #[inline(always)]
    fn is_same_collection_as(&self, _: &Self) -> bool {
        true
    }

    fn successor_state(&self) -> Self {
        Self
    }
}

// threshold
pub(crate) type MemoryReclaimAlways = MemoryReclaimOnThreshold<32>;

/// A `MemoryReclaimPolicy` which reclaims all closed nodes whenever the utilization falls below a threshold.
///
/// The threshold is a function of the constant generic parameter `D`.
/// Specifically, memory of closed nodes will be reclaimed whenever the ratio of closed nodes to all nodes exceeds one over `2^D`.
/// * when `D = 0`: memory will be reclaimed when utilization is below 0.00% (equivalent to never).
/// * when `D = 1`: memory will be reclaimed when utilization is below 50.00%.
/// * when `D = 2`: memory will be reclaimed when utilization is below 75.00%.
/// * when `D = 3`: memory will be reclaimed when utilization is below 87.50%.
/// * when `D = 4`: memory will be reclaimed when utilization is below 93.75%.
/// * ...
#[derive(Default, Clone, Copy)]
pub struct MemoryReclaimOnThreshold<const D: usize>(pub(crate) MemoryState);

// threshold - unidirectional - single-x
impl<'a, const D: usize, V, T> MemoryReclaimPolicy<'a, V, T, NodeRefSingle<'a, V, T>, NodeRefNone>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefSingle<'a, V, T>,
        Next = NodeRefNone,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - unidirectional - vec-x
impl<'a, const D: usize, V, T> MemoryReclaimPolicy<'a, V, T, NodeRefsVec<'a, V, T>, NodeRefNone>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsVec<'a, V, T>,
        Next = NodeRefNone,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - unidirectional - array-x
impl<'a, const D: usize, const N: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefsArray<'a, N, V, T>, NodeRefNone>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsArray<'a, N, V, T>,
        Next = NodeRefNone,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - unidirectional - x-single
impl<'a, const D: usize, V, T> MemoryReclaimPolicy<'a, V, T, NodeRefNone, NodeRefSingle<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefNone,
        Next = NodeRefSingle<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - unidirectional - x-vec
impl<'a, const D: usize, V, T> MemoryReclaimPolicy<'a, V, T, NodeRefNone, NodeRefsVec<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefNone,
        Next = NodeRefsVec<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - unidirectional - x-array
impl<'a, const D: usize, const N: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefNone, NodeRefsArray<'a, N, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefNone,
        Next = NodeRefsArray<'a, N, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - single-single
impl<'a, const D: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefSingle<'a, V, T>, NodeRefSingle<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefSingle<'a, V, T>,
        Next = NodeRefSingle<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - single-vec
impl<'a, const D: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefSingle<'a, V, T>, NodeRefsVec<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefSingle<'a, V, T>,
        Next = NodeRefsVec<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - single-array
impl<'a, const D: usize, const N: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefSingle<'a, V, T>, NodeRefsArray<'a, N, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefSingle<'a, V, T>,
        Next = NodeRefsArray<'a, N, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - vec-single
impl<'a, const D: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefsVec<'a, V, T>, NodeRefSingle<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsVec<'a, V, T>,
        Next = NodeRefSingle<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - vec-vec
impl<'a, const D: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefsVec<'a, V, T>, NodeRefsVec<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsVec<'a, V, T>,
        Next = NodeRefsVec<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - vec-array
impl<'a, const D: usize, const N: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefsVec<'a, V, T>, NodeRefsArray<'a, N, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsVec<'a, V, T>,
        Next = NodeRefsArray<'a, N, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - array-single
impl<'a, const D: usize, const N: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefsArray<'a, N, V, T>, NodeRefSingle<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsArray<'a, N, V, T>,
        Next = NodeRefSingle<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - array-vec
impl<'a, const D: usize, const N: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefsArray<'a, N, V, T>, NodeRefsVec<'a, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsArray<'a, N, V, T>,
        Next = NodeRefsVec<'a, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

// threshold - bidirectional - array-array
impl<'a, const D: usize, const N: usize, const M: usize, V, T>
    MemoryReclaimPolicy<'a, V, T, NodeRefsArray<'a, N, V, T>, NodeRefsArray<'a, M, V, T>>
    for MemoryReclaimOnThreshold<D>
where
    V: Variant<
        'a,
        T,
        Storage = NodeDataLazyClose<T>,
        Prev = NodeRefsArray<'a, N, V, T>,
        Next = NodeRefsArray<'a, M, V, T>,
    >,
    T: 'a,
    V: 'a,
{
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        P: PinnedVec<Node<'a, V, T>> + 'a,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(vec_mut);
        }
    }

    fn is_same_collection_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}
