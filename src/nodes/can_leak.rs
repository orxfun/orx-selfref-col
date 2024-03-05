use crate::{nodes::index::NodeIndex, Node, Variant};
use orx_split_vec::prelude::PinnedVec;

/// Marker trait for types which are safe to leak out of the `SelfRefCol`.
/// In other words, the mutation methods of the collection can only return types implementing `CanLeak`.
///
/// For a given self referential collection `SelfRefCol<'a, V, T, P>`, the following types implement `CanLeak`:
/// * `T` -> values can be taken out of the collection and returned,
/// * `&T` -> immutable references to values can be returned from the collection,
/// * `NodeIndex<'a, V, T>` -> a safe reference to the collection can be returned and used with safety checks to access nodes in constant time.
///
/// On the other hand, neither `Node<'a, V, T>` nor `&Node<'a, V, T>` implements `CanLeak`.
/// Therefore, the nodes and inter-node references cannot leak, they are safely and completely encapsulated inside the `SelfRefCol`.
pub trait CanLeak<'a, V, T, P>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

// T
impl<'a, V, T, P> CanLeak<'a, V, T, P> for T
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, V, T, P> CanLeak<'a, V, T, P> for Option<T>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, V, T, P> CanLeak<'a, V, T, P> for Vec<T>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, const N: usize, V, T, P> CanLeak<'a, V, T, P> for [T; N]
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

// &T
impl<'a, V, T, P> CanLeak<'a, V, T, P> for &T
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, V, T, P> CanLeak<'a, V, T, P> for Option<&T>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, V, T, P> CanLeak<'a, V, T, P> for Vec<&T>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, const N: usize, V, T, P> CanLeak<'a, V, T, P> for [&T; N]
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

// NodeIndex
impl<'a, V, T, P> CanLeak<'a, V, T, P> for NodeIndex<'a, V, T>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, V, T, P> CanLeak<'a, V, T, P> for Option<NodeIndex<'a, V, T>>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, V, T, P> CanLeak<'a, V, T, P> for Vec<NodeIndex<'a, V, T>>
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}

impl<'a, const N: usize, V, T, P> CanLeak<'a, V, T, P> for [NodeIndex<'a, V, T>; N]
where
    V: Variant<'a, T>,
    P: PinnedVec<Node<'a, V, T>>,
{
}
