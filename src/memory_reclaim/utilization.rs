use crate::{Node, NodeDataLazyClose, SelfRefCol, SelfRefColMut, Variant};
use orx_split_vec::prelude::PinnedVec;

impl<'a, V, T, P> SelfRefCol<'a, V, T, P>
where
    V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
    P: PinnedVec<Node<'a, V, T>>,
{
    /// Returns the node utilization as a fraction of active nodes to the used nodes:
    /// * 1.0 when there is no closed node;
    /// * 0.0 when all used memory is used by closed nodes.
    pub fn node_utilization(&self) -> f32 {
        let used = self.pinned_vec.len();
        if used == 0 {
            1.0
        } else {
            let num_occupied = self.len;
            num_occupied as f32 / used as f32
        }
    }
}

impl<'rf, 'a, V, T, P> SelfRefColMut<'rf, 'a, V, T, P>
where
    V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
    P: PinnedVec<Node<'a, V, T>>,
{
    /// Returns the node utilization as a fraction of active nodes to the used nodes:
    /// * 1.0 when there is no closed node;
    /// * 0.0 when all used memory is used by closed nodes.
    #[inline(always)]
    pub fn node_utilization(&self) -> f32 {
        self.col.node_utilization()
    }

    #[inline(always)]
    pub(crate) fn need_to_reclaim_vacant_nodes<const D: usize>(&self) -> bool {
        let used = self.col.pinned_vec.len();
        let allowed_vacant = used >> D;
        let num_vacant = used - self.col.len;
        num_vacant > allowed_vacant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MemoryReclaimNever, NodeDataLazyClose, NodeRefSingle, NodeRefsArray, NodeRefsVec,
        SelfRefCol,
    };
    use float_cmp::approx_eq;

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
    fn node_utilization() {
        let n = 1_000;
        let mut col = SelfRefCol::<Var, _>::new();
        assert!(approx_eq!(f32, col.node_utilization(), 1.0, ulps = 2));

        let values: Vec<_> = (0..n).map(|x| x.to_string()).collect();
        col.mutate(values, |x, values| {
            for val in values {
                _ = x.push_get_ref(val);
            }
            assert!(approx_eq!(f32, x.node_utilization(), 1.0, ulps = 2));
        });

        assert_eq!(col.len(), n);
        assert_eq!(col.pinned_vec.len(), n);

        for i in 0..n {
            col.mutate((n, i), |x, (n, i)| {
                let node = &mut x.col.pinned_vec[i];
                let value = node.data.close();
                x.col.len -= 1;
                assert_eq!(Some(i.to_string()), value);

                let utilization = (n - 1 - i) as f32 / n as f32;
                assert!(approx_eq!(f32, x.node_utilization(), utilization, ulps = 2));
            });
        }

        assert!(approx_eq!(f32, col.node_utilization(), 0.0, ulps = 2));
    }

    #[test]
    fn need_to_reclaim_vacant_nodes() {
        let n = 1_000;
        let mut col = SelfRefCol::<Var, _>::new();

        let values: Vec<_> = (0..n).map(|x| x.to_string()).collect();
        col.mutate(values, |x, values| {
            for val in values {
                _ = x.push_get_ref(val);
            }
            assert!(!x.need_to_reclaim_vacant_nodes::<2>());
        });

        assert_eq!(col.len(), n);
        assert_eq!(col.pinned_vec.len(), n);

        for i in 0..n {
            col.mutate((n, i), |x, (n, i)| {
                let node = &mut x.col.pinned_vec[i];
                let value = node.data.close();
                x.col.len -= 1;
                assert_eq!(Some(i.to_string()), value);

                let threshold = 3 * n / 4;
                let needs_reclaim = (n - 1 - i) < threshold;
                assert_eq!(x.need_to_reclaim_vacant_nodes::<2>(), needs_reclaim);
            });
        }

        assert!(approx_eq!(f32, col.node_utilization(), 0.0, ulps = 2));
    }
}
