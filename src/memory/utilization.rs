/// Node utilization of the underlying storage of the self referential collection.
///
/// The result contains the following bits of information:
/// * `capacity`: number of positions that is already allocated.
/// * `num_active_nodes`: number of active nodes holding data.
/// * `num_closed_nodes`: number of nodes which had been opened and closed afterwards; however, not yet reclaimed.
///
/// Note that `num_active_nodes + num_closed_nodes` reflects the length of the underlying pinned vector,
/// which is less than or equal to the `capacity`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Utilization {
    /// Number of positions that is already allocated.
    pub capacity: usize,
    /// Number of active nodes holding data.
    pub num_active_nodes: usize,
    /// Number of nodes which had been opened and closed afterwards; however, not yet reclaimed.
    pub num_closed_nodes: usize,
}
