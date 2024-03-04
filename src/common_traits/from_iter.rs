use crate::{Node, SelfRefCol, Variant};
use orx_split_vec::prelude::PinnedVec;

impl<'a, V, T, P> FromIterator<T> for SelfRefCol<'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>> + FromIterator<Node<'a, V, T>>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter_nodes = iter.into_iter().map(Node::<'a, V, T>::new_free_node);
        let pinned_vec = P::from_iter(iter_nodes);
        SelfRefCol {
            ends: V::Ends::default(),
            len: pinned_vec.len(),
            pinned_vec,
            memory_reclaim_policy: Default::default(),
            phantom: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use orx_split_vec::{prelude::PinnedVec, Recursive, SplitVec};

    #[derive(Debug, Clone, Copy)]
    struct Var;
    impl<'a> Variant<'a, String> for Var {
        type Storage = NodeDataLazyClose<String>;
        type Prev = NodeRefSingle<'a, Self, String>;
        type Next = NodeRefsVec<'a, Self, String>;
        type Ends = NodeRefsArray<'a, 2, Self, String>;
        type MemoryReclaim = MemoryReclaimNever;
    }

    #[test]
    fn from_iter() {
        let vec = vec![0, 1, 2, 3, 4, 5];
        let iter = vec.into_iter().map(|x| x.to_string());
        let col: SelfRefCol<Var, String, SplitVec<_, Recursive>> = iter.into_iter().collect();

        assert_eq!(6, col.len());
        assert_eq!(0, col.ends().referenced_nodes().count());
        for i in 0..6 {
            let node = &col.pinned_vec.get(i).expect("is-some");

            assert_eq!(0, node.prev().referenced_nodes().count());
            assert_eq!(0, node.next().referenced_nodes().count());

            assert_eq!(node.data(), Some(&i.to_string()));
        }
    }
}
