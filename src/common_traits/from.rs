use crate::{CoreCol, MemoryPolicy, MemoryState, Node, SelfRefCol, Variant};
use orx_pinned_vec::PinnedVec;

impl<V, M, P> From<(P, V::Ends)> for SelfRefCol<V, M, P>
where
    V: Variant,
    M: MemoryPolicy<V>,
    P: PinnedVec<Node<V>>,
{
    fn from(value: (P, V::Ends)) -> Self {
        let (nodes, ends) = value;
        let len = nodes.iter().filter(|x| x.is_active()).count();
        let core = CoreCol::from_raw_parts(nodes, ends, len);
        SelfRefCol::from_raw_parts(core, M::default(), MemoryState::default())
    }
}
