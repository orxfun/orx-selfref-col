use crate::{
    memory_reclaim::memory_state::MemoryState, Node, NodeDataLazyClose, NodeRefNone, NodeRefSingle,
    NodeRefsArray, NodeRefsVec, SelfRefColMut, Variant,
};
use orx_split_vec::prelude::PinnedVec;

/// Policy which determines how the memory of closed nodes will be reclaimed and made useful.
///
/// Two main implementors are:
/// * [`MemoryReclaimOnThreshold<N>`] reclaims unused holes whenever the utilization of the memory falls below a constant threshold determined by `N`.
/// This could be considered as the flexible and general approach.
/// * [`MemoryReclaimNever`] which never reclaims the holes due to popped or removed; i.e., closed, nodes.
/// This approach has the advantage that a `NodeIndex` is never invalidated due to memory reorganization.
/// Note that it still allows to reclaim closed nodes manually.
/// Therefore, it fits very well to situations where
///   * removals from the list are not substantial, or
///   * having valid indices is crucial.
pub trait MemoryReclaimPolicy: Default + Clone + Copy {
    /// Manually attempts to reclaim closed nodes.
    ///
    /// # Safety
    ///
    /// Note that reclaiming closed nodes invalidates node indices (`NodeIndex`) which are already stored outside of this collection.
    fn reclaim_closed_nodes<'rf, 'a, V, T, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        T: 'a,
        V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
        P: PinnedVec<Node<'a, V, T>> + 'a,
        SelfRefColMut<'rf, 'a, V, T, P>: Reclaim<V::Prev, V::Next>;

    /// Returns whether or not two self referential collections are the same memory state or not.
    /// This method is used internally to check validity of `NodeIndex`es.
    fn at_the_same_state_as(&self, collection: &Self) -> bool;

    /// Provides the state succeeding the current memory state.
    fn successor_state(&self) -> Self;
}

// never
/// A do-nothing `MemoryReclaimPolicy` which would never reclaim the memory of the closed nodes, leaving them as holes in the underlying storage.
///
/// This approach has the advantage that a `NodeIndex` is never invalidated due to an automatic memory reorganization.
///
/// Furthermore, node utilization can still be maximized by manually calling `reclaim_closed_nodes` method.
#[derive(Default, Clone, Copy)]
pub struct MemoryReclaimNever(pub(crate) MemoryState);

impl MemoryReclaimPolicy for MemoryReclaimNever {
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, 'a, V, T, P>(_: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        T: 'a,
        V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
        P: PinnedVec<Node<'a, V, T>> + 'a,
        SelfRefColMut<'rf, 'a, V, T, P>: Reclaim<V::Prev, V::Next>,
    {
    }

    #[inline(always)]
    fn at_the_same_state_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    #[inline(always)]
    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
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

impl<const D: usize> MemoryReclaimPolicy for MemoryReclaimOnThreshold<D> {
    #[inline(always)]
    fn reclaim_closed_nodes<'rf, 'a, V, T, P>(vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>)
    where
        T: 'a,
        V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
        P: PinnedVec<Node<'a, V, T>> + 'a,
        SelfRefColMut<'rf, 'a, V, T, P>: Reclaim<V::Prev, V::Next>,
    {
        if vec_mut.need_to_reclaim_vacant_nodes::<D>() {
            vec_mut.reclaim();
        }
    }

    #[inline(always)]
    fn at_the_same_state_as(&self, collection: &Self) -> bool {
        self.0 == collection.0
    }

    #[inline(always)]
    fn successor_state(&self) -> Self {
        Self(self.0.successor_state())
    }
}

/// Trait defining how the `reclaim` operation must be handled so that:
/// * correctness of all inter element references are maintained,
/// * while the collection reclaims the unused nodes, or holes, which occurred due to removals from the collection.
pub trait Reclaim<Prev, Next> {
    /// Reclaims holes due to removals from the collection; i.e., not utilized memory.
    ///
    /// This operation:
    /// * does not lead to allocation of memory or carrying the collection entirely to a different location;
    /// * it might, on the other hand, change positions of a subset of nodes within already allocated memory;
    /// * this eliminates all sparsity and achieves a compact memory layout of nodes.
    ///
    /// All inter element references are restored during the reclaim operation.
    ///
    /// However, node indices (`NodeIndex`) obtained prior to the reclaim operation are invalidated.
    fn reclaim(&mut self);
}

macro_rules! impl_reclaim_unidirectional {
    ($prev:ty, $next:ty) => {
        impl<'rf, 'a, V, T, P> Reclaim<$prev, $next> for SelfRefColMut<'rf, 'a, V, T, P>
        where
            T: 'a,
            V: Variant<'a, T, Storage = NodeDataLazyClose<T>, Prev = $prev, Next = $next>,
            P: PinnedVec<Node<'a, V, T>> + 'a,
        {
            #[inline(always)]
            fn reclaim(&mut self) {
                crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(self);
            }
        }
    };
    ($const1:ident, $prev:ty, $next:ty) => {
        impl<'rf, 'a, const $const1: usize, V, T, P> Reclaim<$prev, $next>
            for SelfRefColMut<'rf, 'a, V, T, P>
        where
            T: 'a,
            V: Variant<'a, T, Storage = NodeDataLazyClose<T>, Prev = $prev, Next = $next>,
            P: PinnedVec<Node<'a, V, T>> + 'a,
        {
            #[inline(always)]
            fn reclaim(&mut self) {
                crate::memory_reclaim::lazy_unidirectional::reclaim_closed_nodes(self);
            }
        }
    };
}

macro_rules! impl_reclaim_bidirectional {
    ($prev:ty, $next:ty) => {
        impl<'rf, 'a, V, T, P> Reclaim<$prev, $next> for SelfRefColMut<'rf, 'a, V, T, P>
        where
            T: 'a,
            V: Variant<'a, T, Storage = NodeDataLazyClose<T>, Prev = $prev, Next = $next>,
            P: PinnedVec<Node<'a, V, T>> + 'a,
        {
            #[inline(always)]
            fn reclaim(&mut self) {
                crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(self);
            }
        }
    };
    ($const1:ident, $prev:ty, $next:ty) => {
        impl<'rf, 'a, const $const1: usize, V, T, P> Reclaim<$prev, $next>
            for SelfRefColMut<'rf, 'a, V, T, P>
        where
            T: 'a,
            V: Variant<'a, T, Storage = NodeDataLazyClose<T>, Prev = $prev, Next = $next>,
            P: PinnedVec<Node<'a, V, T>> + 'a,
        {
            #[inline(always)]
            fn reclaim(&mut self) {
                crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(self);
            }
        }
    };
    ($const1:ident, $const2:ident, $prev:ty, $next:ty) => {
        impl<'rf, 'a, const $const1: usize, const $const2: usize, V, T, P> Reclaim<$prev, $next>
            for SelfRefColMut<'rf, 'a, V, T, P>
        where
            T: 'a,
            V: Variant<'a, T, Storage = NodeDataLazyClose<T>, Prev = $prev, Next = $next>,
            P: PinnedVec<Node<'a, V, T>> + 'a,
        {
            #[inline(always)]
            fn reclaim(&mut self) {
                crate::memory_reclaim::lazy_bidirectional::reclaim_closed_nodes(self);
            }
        }
    };
}

impl_reclaim_unidirectional!(NodeRefNone, NodeRefSingle<'a, V, T>);
impl_reclaim_unidirectional!(NodeRefNone, NodeRefsVec<'a, V, T>);
impl_reclaim_unidirectional!(N, NodeRefNone, NodeRefsArray<'a, N, V, T>);
impl_reclaim_unidirectional!(NodeRefSingle<'a, V, T>, NodeRefNone);
impl_reclaim_unidirectional!(NodeRefsVec<'a, V, T>, NodeRefNone);
impl_reclaim_unidirectional!(N, NodeRefsArray<'a, N, V, T>, NodeRefNone);

impl_reclaim_bidirectional!(NodeRefSingle<'a, V, T>, NodeRefSingle<'a, V, T>);
impl_reclaim_bidirectional!(NodeRefSingle<'a, V, T>, NodeRefsVec<'a, V, T>);
impl_reclaim_bidirectional!(N, NodeRefSingle<'a, V, T>, NodeRefsArray<'a, N, V, T>);

impl_reclaim_bidirectional!(NodeRefsVec<'a, V, T>, NodeRefSingle<'a, V, T>);
impl_reclaim_bidirectional!(NodeRefsVec<'a, V, T>, NodeRefsVec<'a, V, T>);
impl_reclaim_bidirectional!(N, NodeRefsVec<'a, V, T>, NodeRefsArray<'a, N, V, T>);

impl_reclaim_bidirectional!(N, NodeRefsArray<'a, N, V, T>, NodeRefSingle<'a, V, T>);
impl_reclaim_bidirectional!(N, NodeRefsArray<'a, N, V, T>, NodeRefsVec<'a, V, T>);
impl_reclaim_bidirectional!(N, M, NodeRefsArray<'a, N, V, T>, NodeRefsArray<'a, M, V, T>);
