use super::node_refs::NodeRefs;
use crate::{variants::variant::Variant, Node};

/// An empty single node, used to represent cases where the node does not hold any reference.
///
/// The following are some example references which can be expressed by `NodeRefNone`:
///
/// * previous node in a singly linked list;
/// * parent of a node in a tree where bottom up traversal is not necessary.
#[derive(Default, Clone, Debug)]
pub struct NodeRefNone(());

impl<'a, V, T> NodeRefs<'a, V, T> for NodeRefNone
where
    V: Variant<'a, T>,
{
    type References = ();

    #[inline(always)]
    fn new(_: Self::References) -> Self {
        Self(())
    }

    #[inline(always)]
    fn get(&self) -> &Self::References {
        &self.0
    }

    #[inline(always)]
    fn get_mut(&mut self) -> &mut Self::References {
        &mut self.0
    }

    #[inline(always)]
    fn update_reference(&mut self, _: &'a Node<'a, V, T>, _: &'a Node<'a, V, T>) {}

    #[inline(always)]
    fn referenced_nodes(&self) -> impl Iterator<Item = &'a Node<'a, V, T>>
    where
        V: 'a,
        T: 'a,
    {
        std::iter::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryReclaimNever, NodeData, NodeDataLazyClose};

    struct Var;
    impl<'a> Variant<'a, char> for Var {
        type Storage = NodeDataLazyClose<char>;
        type Prev = NodeRefNone;
        type Next = NodeRefNone;
        type Ends = NodeRefNone;
        type MemoryReclaim = MemoryReclaimNever;
    }

    #[test]
    fn new_default() {
        let _new = <NodeRefNone as NodeRefs<'_, Var, char>>::new(());
        let _default = NodeRefNone::default();
    }

    #[test]
    fn get() {
        let nr = <NodeRefNone as NodeRefs<'_, Var, char>>::new(());
        assert_eq!(&(), <NodeRefNone as NodeRefs<'_, Var, char>>::get(&nr));
    }

    #[test]
    fn get_mut() {
        let mut nr = <NodeRefNone as NodeRefs<'_, Var, char>>::new(());
        assert_eq!(
            &mut (),
            <NodeRefNone as NodeRefs<'_, Var, char>>::get_mut(&mut nr)
        );
    }

    #[test]
    fn update_reference() {
        let data = NodeDataLazyClose::active('a');
        let a = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let data = NodeDataLazyClose::active('b');
        let b = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let mut with_ref = <NodeRefNone as NodeRefs<'_, Var, char>>::new(());
        assert_eq!(
            0,
            <NodeRefNone as NodeRefs<'_, Var, char>>::referenced_nodes(&with_ref).count()
        );

        with_ref.update_reference(&a, &b);

        assert_eq!(
            0,
            <NodeRefNone as NodeRefs<'_, Var, char>>::referenced_nodes(&with_ref).count()
        );
    }

    #[test]
    fn referenced_nodes() {
        let with_ref = <NodeRefNone as NodeRefs<'_, Var, char>>::new(());
        assert_eq!(
            0,
            <NodeRefNone as NodeRefs<'_, Var, char>>::referenced_nodes(&with_ref).count()
        );
    }
}
