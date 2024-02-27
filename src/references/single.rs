use std::fmt::Debug;

use super::node_refs::NodeRefs;
use crate::{nodes::node::Node, variants::variant::Variant};

/// A single node reference.
///
/// Underlying reference storage is an optional reference to the single referenced node.
///
/// The following are some example references which can be expressed by `NodeRefSingle`:
///
/// * previous or next node in a linked list;
/// * parent of a node in a tree.
pub struct NodeRefSingle<'a, V, T>(Option<&'a Node<'a, V, T>>)
where
    V: Variant<'a, T>;

impl<'a, V, T> NodeRefs<'a, V, T> for NodeRefSingle<'a, V, T>
where
    V: Variant<'a, T>,
{
    type References = Option<&'a Node<'a, V, T>>;

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
        if let Some(current_reference) = &self.0 {
            if current_reference.ref_eq(prior_reference) {
                self.0 = Some(new_reference);
            }
        }
    }

    #[inline(always)]
    fn referenced_nodes(&self) -> impl Iterator<Item = &'a Node<'a, V, T>>
    where
        V: 'a,
        T: 'a,
    {
        self.0.into_iter()
    }
}

impl<'a, V, T> Default for NodeRefSingle<'a, V, T>
where
    V: Variant<'a, T>,
{
    /// Creates an empty reference.
    #[inline(always)]
    fn default() -> Self {
        Self(None)
    }
}

impl<'a, T: Debug, V> Debug for NodeRefSingle<'a, V, T>
where
    V: Variant<'a, T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NodeRefSingle")
            .field(&self.0.and_then(|x| x.data()))
            .finish()
    }
}

impl<'a, V, T> Clone for NodeRefSingle<'a, V, T>
where
    V: Variant<'a, T>,
{
    /// Clones the reference.
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'a, V, T> From<&'a Node<'a, V, T>> for NodeRefSingle<'a, V, T>
where
    V: Variant<'a, T>,
{
    #[inline(always)]
    fn from(single_reference: &'a Node<'a, V, T>) -> Self {
        Self(Some(single_reference))
    }
}

impl<'a, V, T> From<Option<&'a Node<'a, V, T>>> for NodeRefSingle<'a, V, T>
where
    V: Variant<'a, T>,
{
    #[inline(always)]
    fn from(value: Option<&'a Node<'a, V, T>>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{MemoryReclaimNever, NodeData, NodeDataEagerClose, NodeRefNone};

    struct TestVar;
    impl<'a> Variant<'a, char> for TestVar {
        type Storage = NodeDataEagerClose<char>;
        type Prev = NodeRefNone;
        type Next = NodeRefNone;
        type Ends = NodeRefNone;
        type MemoryReclaim = MemoryReclaimNever;
    }

    #[test]
    fn new_default() {
        let without_ref = NodeRefSingle::<'_, TestVar, _>::new(None);
        assert!(without_ref.get().is_none());

        let without_ref = NodeRefSingle::<'_, TestVar, _>::default();
        assert!(without_ref.get().is_none());

        let without_ref = NodeRefSingle::<'_, TestVar, _>::empty();
        assert!(without_ref.get().is_none());

        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());
        let with_ref = NodeRefSingle::<'_, TestVar, _>::new(Some(&node));
        assert!(with_ref.get().unwrap().ref_eq(&node));
    }

    #[test]
    fn from() {
        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());

        let with_ref: NodeRefSingle<'_, TestVar, _> = (&node).into();
        assert!(with_ref.get().unwrap().ref_eq(&node));

        let with_ref: NodeRefSingle<'_, TestVar, _> = Some(&node).into();
        assert!(with_ref.get().unwrap().ref_eq(&node));
    }

    #[test]
    fn some_none() {
        let without_ref = NodeRefSingle::<'_, TestVar, _>::empty();
        assert!(without_ref.get().is_none());

        let data = NodeDataEagerClose::active('a');
        let node = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());
        let with_ref = NodeRefSingle::<'_, TestVar, _>::from(&node);
        assert!(with_ref.get().unwrap().ref_eq(&node));
    }

    #[test]
    fn get_mut() {
        let mut nr = NodeRefSingle::<'_, TestVar, _>::default();
        assert!(nr.get().is_none());

        let data = NodeDataEagerClose::active('x');
        let x = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());

        *nr.get_mut() = Some(&x);
        assert!(nr.get().unwrap().ref_eq(&x));

        let data = NodeDataEagerClose::active('y');
        let y = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());

        *nr.get_mut() = Some(&y);
        assert!(nr.get().unwrap().ref_eq(&y));

        *nr.get_mut() = None;
        assert!(nr.get().is_none());
    }

    #[test]
    fn update_reference() {
        let data = NodeDataEagerClose::active('a');
        let a = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());

        let data = NodeDataEagerClose::active('b');
        let b = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());

        let mut with_ref: NodeRefSingle<'_, TestVar, _> = Some(&a).into();
        assert!(with_ref.get().unwrap().ref_eq(&a));

        with_ref.update_reference(&b, &a);
        assert!(with_ref.get().unwrap().ref_eq(&a));

        with_ref.update_reference(&a, &b);
        assert!(with_ref.get().unwrap().ref_eq(&b));
    }

    #[test]
    fn referenced_nodes() {
        let data = NodeDataEagerClose::active('a');
        let a = Node::<'_, TestVar, _>::new(data, Default::default(), Default::default());

        let with_ref: NodeRefSingle<'_, TestVar, _> = None.into();

        assert_eq!(0, with_ref.referenced_nodes().count());

        let with_ref: NodeRefSingle<'_, TestVar, _> = Some(&a).into();

        assert_eq!(1, with_ref.referenced_nodes().count());
        let mut iter = with_ref.referenced_nodes();
        assert_eq!(Some(&'a'), iter.next().and_then(|x| x.data()));
        assert!(iter.next().is_none());
    }
}
