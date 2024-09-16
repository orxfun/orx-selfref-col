use crate::{MemoryPolicy, Node, SelfRefCol, Variant};
use orx_pinned_vec::PinnedVec;

impl<V, M, P> FromIterator<V::Item> for SelfRefCol<V, M, P>
where
    V: Variant,
    M: MemoryPolicy<V>,
    P: PinnedVec<Node<V>> + Default,
{
    fn from_iter<I: IntoIterator<Item = V::Item>>(iter: I) -> Self {
        let mut nodes = P::default();
        for data in iter.into_iter() {
            nodes.push(Node::new_free_node(data));
        }
        Self::with_active_nodes(nodes)
    }
}
