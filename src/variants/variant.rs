use super::memory_reclaim::MemoryReclaimPolicy;
use crate::{data::node_data::NodeData, references::node_refs::NodeRefs};

/// Variant defining `SelfRefCol` specifications.
pub trait Variant<'a, T>: Clone + Copy
where
    Self: Sized,
{
    /// The way the data will be stored.
    /// * Values can be stored as a `NodeDataEagerClose`.
    /// In this case, removing an element from the collection literally means removing it from the vector.
    /// * Alternatively, values can be stored as a `NodeDataLazyClose`.
    /// In this case, an element can be removed from the vector by closing the storing node.
    /// Removal can be delayed to allow for better amortized time complexities.
    type Storage: NodeData<T>;

    /// The way the previous node references will be stored.
    /// * `NodeRefNone` if there is no relevant previous node.
    /// * `NodeRefSingle` if there is zero or one relevant previous node.
    /// * `NodeRefsVec` if there is a dynamic number of relevant previous nodes.
    /// * `NodeRefNone` if there is a dynamic number of relevant previous nodes with a constant upper bound.
    type Prev: NodeRefs<'a, Self, T>;

    /// The way the next node references will be stored.
    /// * `NodeRefNone` if there is no relevant previous node.
    /// * `NodeRefSingle` if there is zero or one relevant next node.
    /// * `NodeRefsVec` if there is a dynamic number of relevant next nodes.
    /// * `NodeRefNone` if there is a dynamic number of relevant next nodes with a constant upper bound.
    type Next: NodeRefs<'a, Self, T>;

    /// The way the references to the ends or extremes of the collection will be stored.
    /// * `NodeRefNone` if there is no relevant end.
    /// * `NodeRefSingle` if there is zero or one relevant end node; such as the front of a singly-linked list, or root of a tree.
    /// * `NodeRefsVec` if there is a dynamic number of relevant end nodes.
    /// * `NodeRefNone` if there is a dynamic number of relevant end nodes with a constant upper bound;
    /// such as front and back of a doubly-linked list.
    type Ends: NodeRefs<'a, Self, T>;

    /// The way how memory of closed nodes will be reclaimed:
    /// * `MemoryReclaimNever` will never automatically claim closed nodes.
    /// * `MemoryReclaimOnThreshold<D>` will claim memory of closed nodes whenever the ratio of closed nodes exceeds one over `2^D`.
    type MemoryReclaim: MemoryReclaimPolicy;
}
