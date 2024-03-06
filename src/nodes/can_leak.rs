use crate::{nodes::index::NodeIndex, Node, NodeIndexError, Variant};
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

macro_rules! impl_can_leak {
    ($x:ty) => {
        impl<'a, V, T, P> CanLeak<'a, V, T, P> for $x
        where
            V: Variant<'a, T>,
            P: PinnedVec<Node<'a, V, T>>,
        {
        }

        impl<'a, V, T, P> CanLeak<'a, V, T, P> for Result<$x, NodeIndexError>
        where
            V: Variant<'a, T>,
            P: PinnedVec<Node<'a, V, T>>,
        {
        }
    };
}

macro_rules! impl_can_leak_n {
    ($x:ty) => {
        impl<'a, const N: usize, V, T, P> CanLeak<'a, V, T, P> for $x
        where
            V: Variant<'a, T>,
            P: PinnedVec<Node<'a, V, T>>,
        {
        }

        impl<'a, const N: usize, V, T, P> CanLeak<'a, V, T, P> for Result<$x, NodeIndexError>
        where
            V: Variant<'a, T>,
            P: PinnedVec<Node<'a, V, T>>,
        {
        }
    };
}

impl_can_leak!(T);
impl_can_leak!(Option<T>);
impl_can_leak!(Vec<T>);
impl_can_leak_n!([T; N]);

impl_can_leak!(&T);
impl_can_leak!(Option<&T>);
impl_can_leak!(Vec<&T>);
impl_can_leak_n!([&T; N]);

impl_can_leak!(NodeIndex<'a, V, T>);
impl_can_leak!(Option<NodeIndex<'a, V, T>>);
impl_can_leak!(Vec<NodeIndex<'a, V, T>>);
impl_can_leak_n!([NodeIndex<'a, V, T>; N]);
