use crate::{Node, NodeData, SelfRefCol, Variant};
use orx_split_vec::prelude::PinnedVec;
use std::fmt::Debug;

impl<'a, V, T, P> Debug for SelfRefCol<'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>> + Debug,
    V::Ends: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelfRefCol")
            .field("len", &self.len())
            .field("storage_len", &self.pinned_vec.len())
            .field("ends", &self.ends)
            .field("pinned_vec", &self.pinned_vec)
            .finish()
    }
}

impl<'a, T: Debug, V> Debug for Node<'a, V, T>
where
    V: Variant<'a, T>,
    V::Prev: Debug,
    V::Next: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelfRefNode")
            .field("data", &self.data.get())
            .field("prev", &self.prev)
            .field("next", &self.next)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MemoryReclaimNever, NodeDataLazyClose, NodeRefSingle, NodeRefs, NodeRefsArray, NodeRefsVec,
    };

    #[derive(Debug)]
    struct Var;
    impl<'a> Variant<'a, String> for Var {
        type Storage = NodeDataLazyClose<String>;
        type Prev = NodeRefSingle<'a, Self, String>;
        type Next = NodeRefsVec<'a, Self, String>;
        type Ends = NodeRefsArray<'a, 2, Self, String>;
        type MemoryReclaim = MemoryReclaimNever;
    }

    #[test]
    fn debug_vec() {
        let mut vec = SelfRefCol::<Var, _>::new();

        vec.move_mutate([String::from("a"), String::from("b")], |x, letters| {
            for letter in letters {
                let rf = x.push_get_ref(letter);
                x.set_ends([Some(rf), None]);
            }
        });

        let debug_str = format!("{:?}", vec);
        assert_eq!(debug_str, "SelfRefCol { len: 2, storage_len: 2, ends: NodeRefsVec([Some(\"b\"), None]), pinned_vec: SplitVec [\n    [SelfRefNode { data: Some(\"a\"), prev: NodeRefSingle(None), next: NodeRefsVec([]) }, SelfRefNode { data: Some(\"b\"), prev: NodeRefSingle(None), next: NodeRefsVec([]) }]\n]\n }");

        _ = vec.mutate_take(|x| {
            let first = x.ends().get()[0];
            let data = first.map(|n| n.close_node_take_data(&x));
            x.set_ends([None, None]);
            data
        });

        let debug_str = format!("{:?}", vec);
        assert_eq!(debug_str, "SelfRefCol { len: 1, storage_len: 2, ends: NodeRefsVec([None, None]), pinned_vec: SplitVec [\n    [SelfRefNode { data: Some(\"a\"), prev: NodeRefSingle(None), next: NodeRefsVec([]) }, SelfRefNode { data: None, prev: NodeRefSingle(None), next: NodeRefsVec([]) }]\n]\n }");
    }
}
