use crate::{Node, NodeDataLazyClose, Variant};

impl<'a, V, T> Node<'a, V, T>
where
    V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
{
    /// Creates a new closed node with no data and no prev and next references.
    pub fn closed_node() -> Self {
        Self {
            data: NodeDataLazyClose::closed(),
            prev: Default::default(),
            next: Default::default(),
        }
    }

    /// Returns whether the node is closed or not.
    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        self.data.is_closed()
    }

    /// Returns whether the node is active or not.
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.data.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryReclaimOnThreshold, NodeRefNone, NodeRefSingle, NodeRefs, NodeRefsVec};

    struct Var;
    impl<'a> Variant<'a, char> for Var {
        type Storage = NodeDataLazyClose<char>;
        type Prev = NodeRefSingle<'a, Self, char>;
        type Next = NodeRefsVec<'a, Self, char>;
        type Ends = NodeRefNone;
        type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    }

    #[test]
    fn closed_node() {
        let node = Node::<Var, _>::closed_node();
        assert!(node.prev().get().is_none());
        assert!(node.next().get().is_empty());
        assert!(node.data().is_none());
    }

    #[test]
    fn is_closed_active() {
        let node = Node::<Var, _>::closed_node();
        assert!(node.is_closed());
        assert!(!node.is_active());

        let node = Node::<Var, _>::new_free_node('x');
        assert!(!node.is_closed());
        assert!(node.is_active());
    }
}
