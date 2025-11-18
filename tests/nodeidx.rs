use std::{
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

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

fn hash_single<H: Hash>(val: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn clone() {
    let mut col: Col<String> = SelfRefCol::new();

    let idx1 = push_front(&mut col, 0.to_string());
    let idx2 = push_front(&mut col, 1.to_string());

    assert_ne!(idx1, idx2);

    let cloned = idx1;

    assert_eq!(idx1, cloned);
    assert_ne!(idx2, cloned);
}

#[test]
fn hash() {
    let mut col: Col<String> = SelfRefCol::new();

    let idx1 = push_front(&mut col, 0.to_string());
    let idx2 = push_front(&mut col, 1.to_string());

    let idx1_hash = hash_single(idx1);
    let idx2_hash = hash_single(idx2);

    assert_ne!(idx1_hash, idx2_hash);
}
