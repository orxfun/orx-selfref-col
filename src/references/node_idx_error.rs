use core::fmt::{Debug, Display};

/// Error cases of an invalid node index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIdxError {
    /// RemovedNode => Referenced node is removed from the collection.
    /// Node index can only be used if the corresponding node still belongs to the collection.
    RemovedNode,
    /// OutOfBounds => Node index is does not point to the current nodes of the collection.
    /// This might be due to either of the following:
    /// * the index is being used to access a collection which is different than which it was created for,
    /// * the node that the index is pointing to does not belong to the collection any more due to
    ///   shrinking of the collection.
    OutOfBounds,
    /// ReorganizedCollection => Nodes of the containing collection is re-organized in order to reclaim memory of closed nodes.
    /// Such a reorganization happens:
    /// * after a node removal if the utilization level drops below a threshold on default self-reorganizing memory policies,
    /// * after every removal if the always-reclaim memory is used,
    /// * only if the `reclaim_closed_nodes()` is manually called when never-reclaim policy is used,
    ///   * note that in this case indices are never implicitly invalidated.
    ReorganizedCollection,
}

impl Display for NodeIdxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <NodeIdxError as Debug>::fmt(self, f)
    }
}
