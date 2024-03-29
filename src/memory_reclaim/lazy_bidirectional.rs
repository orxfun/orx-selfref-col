use crate::{
    selfref_col_mut::{into_mut, into_ref},
    Node, NodeData, NodeDataLazyClose, NodeRefs, SelfRefColMut, Variant,
};
use orx_split_vec::prelude::PinnedVec;

pub(crate) fn reclaim_closed_nodes<'rf, 'a, T: 'a, V, P>(
    vec_mut: &mut SelfRefColMut<'rf, 'a, V, T, P>,
) where
    V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
    P: PinnedVec<Node<'a, V, T>> + 'a,
{
    if !vec_mut.col.pinned_vec.is_empty() {
        reorganize_nodes(vec_mut);
        vec_mut.col.memory_reclaimed();

        let len = vec_mut.col.len;
        vec_mut.col.pinned_vec.truncate(len);
    }
}

pub(crate) fn reorganize_nodes<'rf, 'a, T: 'a, V, P>(vec_mut: &SelfRefColMut<'rf, 'a, V, T, P>)
where
    V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
    P: PinnedVec<Node<'a, V, T>> + 'a,
{
    let ends = unsafe { into_mut(&vec_mut.col.ends) };
    let forward = unsafe { into_ref(&vec_mut.col.pinned_vec) }.iter();
    let mut backward = unsafe { into_ref(&vec_mut.col.pinned_vec) }.iter_rev();

    let mut occupied_ptr =
        unsafe { (vec_mut.col.pinned_vec.last_unchecked() as *const Node<'a, V, T>).add(1) };

    for vacant in forward {
        if vacant.ref_eq_to_ptr(occupied_ptr) {
            // forward and backward iterators converged; search may stop
            break;
        }
        if vacant.is_closed() {
            let mut swapped = false;
            for occupied in backward.by_ref() {
                if occupied.is_active() {
                    occupied_ptr = occupied as *const Node<'a, V, T>;
                    debug_assert!(
                        vec_mut.col.pinned_vec.index_of(vacant).expect("is-some")
                            < vec_mut.col.pinned_vec.index_of(occupied).expect("is-some")
                    );

                    // found an active node, which cannot be the closed vacant node
                    swap(vacant, occupied);
                    ends.update_reference(occupied, vacant);
                    swapped = true;
                    break;
                } else if occupied.ref_eq(vacant) {
                    // forward and backward iterators converged; search may stop
                    return;
                }
            }

            debug_assert!(swapped);
        }
    }
}

fn swap<'a, T: 'a, V>(vacant: &'a Node<'a, V, T>, occupied: &'a Node<'a, V, T>)
where
    V: Variant<'a, T, Storage = NodeDataLazyClose<T>>,
{
    debug_assert!(vacant.is_closed());
    debug_assert!(occupied.is_active());

    let vacant_mut = std::hint::black_box(unsafe { into_mut(vacant) });
    let occupied_mut = std::hint::black_box(unsafe { into_mut(occupied) });

    let value = occupied_mut.data.close().expect("is active");
    vacant_mut.data = NodeDataLazyClose::active(value);

    vacant_mut.next = occupied.next.clone();
    occupied_mut.next = V::Next::empty();
    for next in vacant.next.referenced_nodes() {
        let next_prev = &mut unsafe { into_mut(next) }.prev;
        next_prev.update_reference(occupied, vacant);
    }

    vacant_mut.prev = occupied.prev.clone();
    occupied_mut.prev = V::Prev::empty();
    for prev in vacant.prev.referenced_nodes() {
        let prev_next = &mut unsafe { into_mut(prev) }.next;
        prev_next.update_reference(occupied, vacant);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryReclaimNever, NodeDataLazyClose, NodeRefSingle, NodeRefsArray, SelfRefCol};
    use test_case::test_case;

    #[derive(Debug, Clone, Copy)]
    struct Var;
    impl<'a> Variant<'a, String> for Var {
        type Storage = NodeDataLazyClose<String>;
        type Prev = NodeRefSingle<'a, Self, String>;
        type Next = NodeRefSingle<'a, Self, String>;
        type Ends = NodeRefsArray<'a, 2, Self, String>;
        type MemoryReclaim = MemoryReclaimNever;
    }

    fn new_full_col<'a>(n: usize) -> SelfRefCol<'a, Var, String> {
        let mut col = SelfRefCol::<Var, _>::new();

        let values: Vec<_> = (0..n).map(|x| x.to_string()).collect();
        col.mutate(values, |x, values| {
            for val in values {
                _ = x.push_get_ref(val);
            }
        });

        col
    }

    fn new_col<'a>(n: usize, vacant_indices: Vec<usize>) -> SelfRefCol<'a, Var, String> {
        let mut col = new_full_col(n);

        col.mutate(vacant_indices, |x, indices| {
            for i in indices {
                let node = &mut x.col.pinned_vec[i];
                let _ = node.data.close();
                x.col.len -= 1;
            }
        });

        col
    }

    fn num_occupied(vec: &SelfRefCol<Var, String>) -> usize {
        vec.pinned_vec.iter().filter(|x| x.is_active()).count()
    }
    fn num_vacant(vec: &SelfRefCol<Var, String>) -> usize {
        vec.pinned_vec.iter().filter(|x| x.is_closed()).count()
    }

    #[test_case(1)]
    #[test_case(2)]
    #[test_case(3)]
    #[test_case(16)]
    #[test_case(254)]
    #[test_case(987)]
    #[test_case(3254)]
    fn when_full(n: usize) {
        let mut col = new_full_col(n);

        assert_eq!(num_occupied(&col), n);
        assert_eq!(num_vacant(&col), 0);

        let mut x = SelfRefColMut::new(&mut col);
        reclaim_closed_nodes(&mut x);

        assert_eq!(num_occupied(&col), n);
        assert_eq!(num_vacant(&col), 0);
    }

    #[test_case(1)]
    #[test_case(2)]
    #[test_case(3)]
    #[test_case(16)]
    #[test_case(254)]
    #[test_case(987)]
    #[test_case(3254)]
    fn when_one_vacant_at_end(n: usize) {
        let mut col = new_col(n, vec![n - 1]);

        assert_eq!(num_occupied(&col), n - 1);
        assert_eq!(num_vacant(&col), 1);

        let mut x = SelfRefColMut::new(&mut col);
        reclaim_closed_nodes(&mut x);

        assert_eq!(num_occupied(&col), n - 1);
        assert_eq!(num_vacant(&col), 0);
    }

    #[test_case(1)]
    #[test_case(2)]
    #[test_case(3)]
    #[test_case(16)]
    #[test_case(254)]
    #[test_case(987)]
    #[test_case(3254)]
    fn when_one_vacant_in_middle(n: usize) {
        let mut col = new_col(n, vec![n / 2]);

        assert_eq!(num_occupied(&col), n - 1);
        assert_eq!(num_vacant(&col), 1);

        let mut x = SelfRefColMut::new(&mut col);
        reclaim_closed_nodes(&mut x);

        assert_eq!(num_occupied(&col), n - 1);
        assert_eq!(num_vacant(&col), 0);
    }

    #[test_case(1)]
    #[test_case(2)]
    #[test_case(3)]
    #[test_case(16)]
    #[test_case(254)]
    #[test_case(987)]
    #[test_case(3254)]
    fn when_one_vacant_at_start(n: usize) {
        let mut col = new_col(n, vec![0]);

        assert_eq!(num_occupied(&col), n - 1);
        assert_eq!(num_vacant(&col), 1);

        let mut x = SelfRefColMut::new(&mut col);
        reclaim_closed_nodes(&mut x);

        assert_eq!(num_occupied(&col), n - 1);
        assert_eq!(num_vacant(&col), 0);
    }

    #[test_case(0)]
    #[test_case(1)]
    #[test_case(2)]
    #[test_case(3)]
    #[test_case(16)]
    #[test_case(254)]
    #[test_case(987)]
    #[test_case(3254)]
    fn when_half_is_closed(n: usize) {
        let dropped_indices: Vec<_> = (0..n).filter(|x| x % 2 == 0).collect();
        let num_dropped = dropped_indices.len();
        let mut col = new_col(n, dropped_indices);

        assert_eq!(num_occupied(&col), n - num_dropped);
        assert_eq!(num_vacant(&col), num_dropped);

        let mut x = SelfRefColMut::new(&mut col);
        reclaim_closed_nodes(&mut x);

        assert_eq!(num_occupied(&col), n - num_dropped);
        assert_eq!(num_vacant(&col), 0);
    }
}
