use std::marker::PhantomData;

use orx_selfref_col::{
    MemoryPolicy, MemoryReclaimNever, Node, NodeIdx, RefsNone, RefsSingle, SelfRefCol, Variant,
};
use orx_split_vec::{Recursive, SplitVec};

struct Singly<T>(PhantomData<T>);

impl<T> Variant for Singly<T> {
    type Item = T;

    type Prev = RefsNone;

    type Next = RefsSingle<Self>;

    type Ends = RefsSingle<Self>;
}

type Col<T, M = MemoryReclaimNever> =
    SelfRefCol<Singly<T>, M, SplitVec<Node<Singly<T>>, Recursive>>;

fn push_front<M>(col: &mut Col<String, M>, value: String) -> NodeIdx<Singly<String>>
where
    M: MemoryPolicy<Singly<String>>,
{
    let idx = col.push(value);

    if let Some(old_front) = col.ends().get().cloned() {
        col.node_mut(&idx).next_mut().set(Some(old_front));
    }

    col.ends_mut().set(Some(idx.clone()));

    NodeIdx::new(col.memory_state(), &idx)
}

#[test]
fn clone() {
    let mut col: Col<String> = SelfRefCol::new();

    let idx1 = push_front(&mut col, 0.to_string());
    let idx2 = push_front(&mut col, 1.to_string());

    assert_ne!(idx1, idx2);

    let cloned = idx1.clone();

    assert_eq!(idx1, cloned);
    assert_ne!(idx2, cloned);
}
