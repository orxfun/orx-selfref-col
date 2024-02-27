use std::fmt::Debug;

use super::node_refs::NodeRefs;
use crate::{nodes::node::Node, variants::variant::Variant};

/// A constant-bounded number of node references.
///
/// Underlying reference storage is an array of optional references to the referenced nodes.
///
/// The following are some example references which can be expressed by `NodeRefsVec`:
///
/// * children of a node in a binary, or quaternary, etc., tree;
/// * head (tail) nodes of edges outgoing from (incoming to) a node in a graph with bounded out degree (in degree) per node.
pub struct NodeRefsArray<'a, const N: usize, V, T>([Option<&'a Node<'a, V, T>>; N])
where
    V: Variant<'a, T>;

impl<'a, const N: usize, V, T> NodeRefs<'a, V, T> for NodeRefsArray<'a, N, V, T>
where
    V: Variant<'a, T>,
{
    type References = [Option<&'a Node<'a, V, T>>; N];

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
        for item in self.0.iter_mut() {
            if let Some(current_reference) = &item {
                if current_reference.ref_eq(prior_reference) {
                    *item = Some(new_reference);
                }
            }
        }
    }

    fn referenced_nodes(&self) -> impl Iterator<Item = &'a Node<'a, V, T>>
    where
        V: 'a,
        T: 'a,
    {
        self.0.iter().flat_map(|x| x.iter()).copied()
    }
}

impl<'a, const N: usize, V, T> Default for NodeRefsArray<'a, N, V, T>
where
    V: Variant<'a, T>,
{
    /// Creates an array of None references.
    #[inline(always)]
    fn default() -> Self {
        Self([None; N])
    }
}

impl<'a, const N: usize, T: Debug, V> Debug for NodeRefsArray<'a, N, V, T>
where
    V: Variant<'a, T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NodeRefsVec")
            .field(
                &self
                    .0
                    .iter()
                    .map(|x| x.and_then(|x| x.data()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a, const N: usize, V, T> Clone for NodeRefsArray<'a, N, V, T>
where
    V: Variant<'a, T>,
{
    /// Clones the reference.
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'a, const N: usize, V, T> From<[Option<&'a Node<'a, V, T>>; N]> for NodeRefsArray<'a, N, V, T>
where
    V: Variant<'a, T>,
{
    #[inline(always)]
    fn from(array_of_references: [Option<&'a Node<'a, V, T>>; N]) -> Self {
        Self::new(array_of_references)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        let without_ref = NodeRefsArray::<'_, 2, Var, _>::new([None, None]);
        assert!(without_ref.get()[0].is_none());
        assert!(without_ref.get()[1].is_none());

        let without_ref = NodeRefsArray::<'_, 2, Var, _>::default();
        assert!(without_ref.get()[0].is_none());
        assert!(without_ref.get()[1].is_none());

        let without_ref = NodeRefsArray::<'_, 2, Var, _>::empty();
        assert!(without_ref.get()[0].is_none());
        assert!(without_ref.get()[1].is_none());

        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, Var, _>::new(data, Default::default(), Default::default());
        let with_ref = NodeRefsArray::<'_, 2, Var, _>::new([Some(&node), None]);
        assert!(with_ref.get()[0].unwrap().ref_eq(&node));
        assert!(with_ref.get()[1].is_none());
    }

    #[test]
    fn clone() {
        let without_ref = NodeRefsArray::<'_, 2, Var, _>::new([None, None]);
        assert!(without_ref.clone().get()[0].is_none());
        assert!(without_ref.clone().get()[1].is_none());

        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, Var, _>::new(data, Default::default(), Default::default());
        let with_ref = NodeRefsArray::<'_, 2, Var, _>::new([Some(&node), None]);
        assert!(with_ref.clone().get()[0].unwrap().ref_eq(&node));
        assert!(with_ref.clone().get()[1].is_none());
    }

    #[test]
    fn from() {
        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, Var, _>::new(data, Default::default(), Default::default());
        let with_ref: NodeRefsArray<'_, 2, Var, _> = [Some(&node), None].into();
        assert!(with_ref.get()[0].unwrap().ref_eq(&node));
        assert!(with_ref.get()[1].is_none());
    }

    #[test]
    fn get_mut() {
        let mut nr = NodeRefsArray::<'_, 2, Var, _>::new([None, None]);
        assert!(nr.get()[0].is_none());
        assert!(nr.get()[1].is_none());

        let data = NodeDataEagerClose::active('x');
        let x = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        nr.get_mut()[0] = Some(&x);

        assert!(nr.get()[0].unwrap().ref_eq(&x));
        assert!(nr.get()[1].is_none());

        let data = NodeDataEagerClose::active('y');
        let y = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        nr.get_mut()[1] = Some(&y);

        assert!(nr.get()[0].unwrap().ref_eq(&x));
        assert!(nr.get()[1].unwrap().ref_eq(&y));

        nr.get_mut()[0] = None;

        assert!(nr.get()[0].is_none());
        assert!(nr.get()[1].unwrap().ref_eq(&y));

        nr.get_mut()[1] = None;
        assert!(nr.get()[0].is_none());
        assert!(nr.get()[1].is_none());
    }

    #[test]
    fn update_reference() {
        let data = NodeDataEagerClose::active('a');
        let a = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let data = NodeDataEagerClose::active('b');
        let b = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let mut with_ref: NodeRefsArray<'_, 2, Var, _> = [Some(&a), None].into();
        assert!(with_ref.get()[0].unwrap().ref_eq(&a));
        assert!(with_ref.get()[1].is_none());

        with_ref.update_reference(&b, &a);
        assert!(with_ref.get()[0].unwrap().ref_eq(&a));
        assert!(with_ref.get()[1].is_none());

        with_ref.update_reference(&a, &b);
        assert!(with_ref.get()[0].unwrap().ref_eq(&b));
        assert!(with_ref.get()[1].is_none());
    }

    #[test]
    fn referenced_nodes() {
        let data = NodeDataEagerClose::active('a');
        let a = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let data = NodeDataEagerClose::active('b');
        let b = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let data = NodeDataEagerClose::active('c');
        let c = Node::<'_, Var, _>::new(data, Default::default(), Default::default());

        let with_ref: NodeRefsArray<'_, 3, Var, _> = [Some(&a), None, Some(&c)].into();

        assert_eq!(2, with_ref.referenced_nodes().count());
        let mut iter = with_ref.referenced_nodes();
        assert_eq!(Some(&'a'), iter.next().and_then(|x| x.data()));
        assert_eq!(Some(&'c'), iter.next().and_then(|x| x.data()));
        assert!(iter.next().is_none());

        let with_ref: NodeRefsArray<'_, 3, Var, _> = [Some(&a), Some(&b), Some(&c)].into();

        assert_eq!(3, with_ref.referenced_nodes().count());
        let mut iter = with_ref.referenced_nodes();
        assert_eq!(Some(&'a'), iter.next().and_then(|x| x.data()));
        assert_eq!(Some(&'b'), iter.next().and_then(|x| x.data()));
        assert_eq!(Some(&'c'), iter.next().and_then(|x| x.data()));
        assert!(iter.next().is_none());
    }
}
