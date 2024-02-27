use std::fmt::Debug;

use super::node_refs::NodeRefs;
use crate::{nodes::node::Node, variants::variant::Variant};

/// A dynamic number of node references.
///
/// Underlying reference storage is a vector of references to the referenced nodes.
///
/// The following are some example references which can be expressed by `NodeRefsVec`:
///
/// * children of a node in a tree;
/// * head (tail) nodes of edges outgoing from (incoming to) a node in a graph.
pub struct NodeRefsVec<'a, V, T>(Vec<&'a Node<'a, V, T>>)
where
    V: Variant<'a, T>;

impl<'a, V, T> NodeRefs<'a, V, T> for NodeRefsVec<'a, V, T>
where
    V: Variant<'a, T>,
{
    type References = Vec<&'a Node<'a, V, T>>;

    #[inline(always)]
    fn new(references: Self::References) -> Self {
        Self(references)
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
    fn update_reference(
        &mut self,
        prior_reference: &'a Node<'a, V, T>,
        new_reference: &'a Node<'a, V, T>,
    ) {
        for current_reference in self.0.iter_mut() {
            if current_reference.ref_eq(prior_reference) {
                *current_reference = new_reference;
            }
        }
    }

    fn referenced_nodes(&self) -> impl Iterator<Item = &'a Node<'a, V, T>>
    where
        V: 'a,
        T: 'a,
    {
        self.0.iter().copied()
    }
}

impl<'a, V, T> Default for NodeRefsVec<'a, V, T>
where
    V: Variant<'a, T>,
{
    /// Creates an empty vector of references.
    #[inline(always)]
    fn default() -> Self {
        Self(vec![])
    }
}

impl<'a, T: Debug, V> Debug for NodeRefsVec<'a, V, T>
where
    V: Variant<'a, T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NodeRefsVec")
            .field(&self.0.iter().map(|x| x.data()).collect::<Vec<_>>())
            .finish()
    }
}

impl<'a, V, T> Clone for NodeRefsVec<'a, V, T>
where
    V: Variant<'a, T>,
{
    /// Clones the reference.
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, V, T> From<Vec<&'a Node<'a, V, T>>> for NodeRefsVec<'a, V, T>
where
    V: Variant<'a, T>,
{
    #[inline(always)]
    fn from(vec_of_references: Vec<&'a Node<'a, V, T>>) -> Self {
        Self::new(vec_of_references)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryReclaimNever, NodeData, NodeDataEagerClose, NodeRefNone};

    struct Var;
    impl<'a> Variant<'a, char> for Var {
        type Storage = NodeDataEagerClose<char>;
        type Prev = NodeRefNone;
        type Next = NodeRefNone;
        type Ends = NodeRefNone;
        type MemoryReclaim = MemoryReclaimNever;
    }

    #[test]
    fn new_default() {
        let without_ref = NodeRefsVec::<'_, Var, _>::new(vec![]);
        assert!(without_ref.get().is_empty());

        let without_ref = NodeRefsVec::<'_, Var, _>::default();
        assert!(without_ref.get().is_empty());

        let without_ref = NodeRefsVec::<'_, Var, _>::empty();
        assert!(without_ref.get().is_empty());

        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, Var, _>::new(data, Default::default(), Default::default());
        let with_ref = NodeRefsVec::<'_, Var, _>::new(vec![&node]);
        assert_eq!(1, with_ref.get().len());
        assert!(with_ref.get()[0].ref_eq(&node));
    }

    #[test]
    fn clone() {
        let without_ref = NodeRefsVec::<'_, Var, _>::new(vec![]);
        assert!(without_ref.clone().get().is_empty());

        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, Var, _>::new(data, Default::default(), Default::default());
        let with_ref = NodeRefsVec::<'_, Var, _>::new(vec![&node]);
        assert_eq!(1, with_ref.clone().get().len());
        assert!(with_ref.clone().get()[0].ref_eq(&node));
    }

    #[test]
    fn from() {
        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, Var, _>::new(data, Default::default(), Default::default());
        let with_ref: NodeRefsVec<'_, Var, _> = vec![&node].into();
        assert_eq!(1, with_ref.get().len());
        assert!(with_ref.get()[0].ref_eq(&node));
    }

    #[test]
    fn get_mut() {
        let mut nr = NodeRefsVec::<'_, Var, _>::new(vec![]);
        assert!(nr.get().is_empty());

        let data = NodeDataEagerClose::active('x');
        let x = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        nr.get_mut().push(&x);

        assert_eq!(1, nr.get().len());
        assert!(nr.get()[0].ref_eq(&x));

        let data = NodeDataEagerClose::active('y');
        let y = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        nr.get_mut().push(&y);

        assert_eq!(2, nr.get().len());
        assert!(nr.get()[0].ref_eq(&x));
        assert!(nr.get()[1].ref_eq(&y));

        nr.get_mut().remove(0);

        assert_eq!(1, nr.get().len());
        assert!(nr.get()[0].ref_eq(&y));

        nr.get_mut().clear();
        assert!(nr.get().is_empty());
    }

    #[test]
    fn update_reference() {
        let data = NodeDataEagerClose::active('a');
        let a = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let data = NodeDataEagerClose::active('b');
        let b = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let mut with_ref: NodeRefsVec<'_, Var, _> = vec![&a].into();
        assert!(with_ref.get()[0].ref_eq(&a));

        with_ref.update_reference(&b, &a);
        assert!(with_ref.get()[0].ref_eq(&a));

        with_ref.update_reference(&a, &b);
        assert!(with_ref.get()[0].ref_eq(&b));
    }

    #[test]
    fn referenced_nodes() {
        let data = NodeDataEagerClose::active('a');
        let a = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let data = NodeDataEagerClose::active('b');
        let b = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let data = NodeDataEagerClose::active('c');
        let c = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let with_ref: NodeRefsVec<'_, Var, _> = vec![&a, &c].into();

        assert_eq!(2, with_ref.referenced_nodes().count());
        let mut iter = with_ref.referenced_nodes();
        assert_eq!(Some(&'a'), iter.next().and_then(|x| x.data()));
        assert_eq!(Some(&'c'), iter.next().and_then(|x| x.data()));
        assert!(iter.next().is_none());

        let with_ref: NodeRefsVec<'_, Var, _> = vec![&a, &b, &c].into();

        assert_eq!(3, with_ref.referenced_nodes().count());
        let mut iter = with_ref.referenced_nodes();
        assert_eq!(Some(&'a'), iter.next().and_then(|x| x.data()));
        assert_eq!(Some(&'b'), iter.next().and_then(|x| x.data()));
        assert_eq!(Some(&'c'), iter.next().and_then(|x| x.data()));
        assert!(iter.next().is_none());
    }
}
