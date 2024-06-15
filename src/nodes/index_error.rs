use std::{
    error::Error,
    fmt::{Debug, Display},
};

/// Error observed while using a `NodeIndex` to access a node in a `SelfRefCol`.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum NodeIndexError {
    /// RemovedNode => Referenced node is removed from the collection. Node index can only be used if the corresponding node still belongs to the collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_selfref_col::*;
    ///
    /// #[derive(Clone, Copy, Debug)]
    /// struct Var;
    /// impl<'a> Variant<'a, char> for Var {
    ///     type Storage = NodeDataLazyClose<char>;
    ///     type Prev = NodeRefSingle<'a, Self, char>;
    ///     type Next = NodeRefsVec<'a, Self, char>;
    ///     type Ends = NodeRefsArray<'a, 2, Self, char>;
    ///     type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    /// }
    ///
    /// let mut col = SelfRefCol::<Var, _>::new();
    /// let [a, b, c, d, e, f, g] = col
    ///     .mutate_take(['a', 'b', 'c', 'd', 'e', 'f', 'g'], |x, values| {
    ///         values.map(|val| x.push_get_ref(val).index(&x))
    ///     });
    ///
    /// let removed_b = col.mutate_take(b, |x, b| x.as_node_ref(b).close_node_take_data(&x)); // does not trigger reclaim yet
    /// assert_eq!(removed_b, 'b');
    ///
    /// assert_eq!(a.invalidity_reason_for_collection(&col), None);
    /// assert_eq!(c.invalidity_reason_for_collection(&col), None);
    ///
    /// assert_eq!(b.invalidity_reason_for_collection(&col), Some(NodeIndexError::RemovedNode));
    /// ```
    RemovedNode,
    /// WrongCollection => Node index is used on a collection different than the collection it is created for.  Node indices can only be used for the collection they belong to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_selfref_col::*;
    ///
    /// #[derive(Clone, Copy, Debug)]
    /// struct Var;
    /// impl<'a> Variant<'a, char> for Var {
    ///     type Storage = NodeDataLazyClose<char>;
    ///     type Prev = NodeRefSingle<'a, Self, char>;
    ///     type Next = NodeRefsVec<'a, Self, char>;
    ///     type Ends = NodeRefsArray<'a, 2, Self, char>;
    ///     type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    /// }
    ///
    /// let mut col1 = SelfRefCol::<Var, _>::new();
    /// let a = col1.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));
    ///
    /// let col2 = SelfRefCol::<Var, _>::new();
    ///
    /// assert_eq!(a.invalidity_reason_for_collection(&col1), None);
    /// assert_eq!(a.invalidity_reason_for_collection(&col2), Some(NodeIndexError::WrongCollection));
    /// ```
    WrongCollection,
    /// ReorganizedCollection => All nodes of the containing collection is re-organized in order to reclaim memory of closed nodes. Such a reorganization happens only if the collection uses `MemoryReclaimOnThreshold` policy and utilization level of memory drops below the threshold due to pop and remove operations. It is never observed if the list only grows or if `MemoryReclaimNever` policy is used. In this case, the references need to be recreated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_selfref_col::*;
    ///
    /// #[derive(Clone, Copy, Debug)]
    /// struct Var;
    /// impl<'a> Variant<'a, char> for Var {
    ///     type Storage = NodeDataLazyClose<char>;
    ///     type Prev = NodeRefSingle<'a, Self, char>;
    ///     type Next = NodeRefsVec<'a, Self, char>;
    ///     type Ends = NodeRefsArray<'a, 2, Self, char>;
    ///     type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    /// }
    ///
    /// let mut col = SelfRefCol::<Var, _>::new();
    /// let [a, b, c] = col.mutate_take(['a', 'b', 'c'], |x, values| {
    ///     values.map(|val| x.push_get_ref(val).index(&x))
    /// });
    ///
    /// // triggers memory reclaim, invalidating all prior node indices
    /// let removed_b = col.mutate_take(b, |x, b| x.as_node_ref(b).close_node_take_data(&x));
    /// assert_eq!(removed_b, 'b');
    ///
    /// assert_eq!(a.invalidity_reason_for_collection(&col), Some(NodeIndexError::ReorganizedCollection));
    /// assert_eq!(b.invalidity_reason_for_collection(&col), Some(NodeIndexError::ReorganizedCollection));
    /// assert_eq!(c.invalidity_reason_for_collection(&col), Some(NodeIndexError::ReorganizedCollection));
    /// ```
    ReorganizedCollection,
}

impl Error for NodeIndexError {}

impl Debug for NodeIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RemovedNode => write!(f,"RemovedNode => Referenced node is removed from the collection. Node index can only be used if the corresponding node still belongs to the collection."),
            Self::WrongCollection => write!(f, "WrongCollection => Node index is used on a collection different than the collection it is created for.  Node indices can only be used for the collection they belong to."),
            Self::ReorganizedCollection => write!(f, "ReorganizedCollection => All nodes of the containing collection is re-organized in order to reclaim memory of closed nodes. Such a reorganization happens only if the collection uses `MemoryReclaimOnThreshold` policy and utilization level of memory drops below the threshold due to pop and remove operations. It is never observed if the list only grows or if `MemoryReclaimNever` policy is used. In this case, the references need to be recreated."),
        }
    }
}

impl Display for NodeIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <NodeIndexError as Debug>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug() {
        assert_eq!(&format!("{:?}", NodeIndexError::RemovedNode),
            "RemovedNode => Referenced node is removed from the collection. Node index can only be used if the corresponding node still belongs to the collection.");
        assert_eq!(&format!("{}", NodeIndexError::RemovedNode),
            "RemovedNode => Referenced node is removed from the collection. Node index can only be used if the corresponding node still belongs to the collection.");

        assert_eq!(&format!("{:?}", NodeIndexError::WrongCollection),
            "WrongCollection => Node index is used on a collection different than the collection it is created for.  Node indices can only be used for the collection they belong to.");
        assert_eq!(&format!("{}", NodeIndexError::WrongCollection),
            "WrongCollection => Node index is used on a collection different than the collection it is created for.  Node indices can only be used for the collection they belong to.");

        assert_eq!(&format!("{:?}", NodeIndexError::ReorganizedCollection),
            "ReorganizedCollection => All nodes of the containing collection is re-organized in order to reclaim memory of closed nodes. Such a reorganization happens only if the collection uses `MemoryReclaimOnThreshold` policy and utilization level of memory drops below the threshold due to pop and remove operations. It is never observed if the list only grows or if `MemoryReclaimNever` policy is used. In this case, the references need to be recreated.");
        assert_eq!(&format!("{}", NodeIndexError::ReorganizedCollection),
            "ReorganizedCollection => All nodes of the containing collection is re-organized in order to reclaim memory of closed nodes. Such a reorganization happens only if the collection uses `MemoryReclaimOnThreshold` policy and utilization level of memory drops below the threshold due to pop and remove operations. It is never observed if the list only grows or if `MemoryReclaimNever` policy is used. In this case, the references need to be recreated.");
    }
}
