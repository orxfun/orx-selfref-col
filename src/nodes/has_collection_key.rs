use crate::{Node, SelfRefColMut, SelfRefColVisit, Variant};
use orx_split_vec::prelude::PinnedVec;

pub trait HasCollectionKey<'a, V, T>
where
    V: Variant<'a, T>,
{
    fn collection_key(&self) -> V::MemoryReclaim;
}

impl<'rf, 'a, V, T, P> HasCollectionKey<'a, V, T> for SelfRefColVisit<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    fn collection_key(&self) -> V::MemoryReclaim {
        self.col.memory_reclaim_policy
    }
}

impl<'rf, 'a, V, T, P> HasCollectionKey<'a, V, T> for SelfRefColMut<'rf, 'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
    fn collection_key(&self) -> V::MemoryReclaim {
        self.col.memory_reclaim_policy
    }
}
