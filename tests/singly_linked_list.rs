use orx_iterable::Collection;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::*;
use orx_split_vec::{Recursive, SplitVec};
use std::marker::PhantomData;

struct Singly<T>(PhantomData<T>);

type PolicyNever = MemoryReclaimNever;
type PolicyOnThreshold<const D: usize, T> =
    MemoryReclaimOnThreshold<D, Singly<T>, OnThresholdReclaimer>;

#[derive(Clone, Default)]
pub struct OnThresholdReclaimer;
impl<T> MemoryReclaimer<Singly<T>> for OnThresholdReclaimer {
    fn reclaim_nodes<P>(col: &mut CoreCol<Singly<T>, P>) -> bool
    where
        P: PinnedVec<Node<Singly<T>>>,
    {
        let mut nodes_moved = false;

        if let Some(mut current) = col.ends().get().cloned() {
            let mut prev = None;

            for vacant in 0..col.nodes().len() {
                if col.nodes()[vacant].is_active() {
                    continue;
                }

                loop {
                    let occupied = col.position_of_unchecked(&current);

                    let swapped = occupied > vacant;

                    if swapped {
                        nodes_moved = true;
                        swap(col, vacant, occupied, prev);
                    }

                    match col.node(&current).next().get().cloned() {
                        Some(next) => {
                            prev = Some(occupied);
                            current = next;
                        }
                        None => return nodes_moved,
                    }

                    if swapped {
                        break;
                    }
                }
            }
        }

        nodes_moved
    }
}

fn swap<P, T>(col: &mut CoreCol<Singly<T>, P>, vacant: usize, occupied: usize, prev: Option<usize>)
where
    P: PinnedVec<Node<Singly<T>>>,
{
    let new_idx = col.node_ptr_at_pos(vacant);
    // let old_idx = col.node_idx_at(occupied);

    match prev {
        Some(prev) => col.nodes_mut()[prev].next_mut().set(Some(new_idx)),
        None => col.ends_mut().set(Some(new_idx)), // must be the front
    }

    col.move_node(vacant, occupied);
}

impl<T> Variant for Singly<T> {
    type Item = T;

    type Prev = RefsNone;

    type Next = RefsSingle<Self>;

    type Ends = RefsSingle<Self>;
}

type Col<T, M> = SelfRefCol<Singly<T>, M, SplitVec<Node<Singly<T>>, Recursive>>;

fn to_str(numbers: &[usize]) -> Vec<String> {
    numbers.iter().map(|x| x.to_string()).collect()
}

fn front<M>(col: &Col<String, M>) -> &Node<Singly<String>>
where
    M: MemoryPolicy<Singly<String>>,
{
    col.node(col.ends().get().unwrap())
}

fn forward<M>(col: &Col<String, M>) -> Vec<String>
where
    M: MemoryPolicy<Singly<String>>,
{
    let mut vec = vec![];

    if !col.is_empty() {
        let front = front(col);
        vec.push(front.data().unwrap().clone());

        let mut current = front;

        while let Some(next) = current.next().get() {
            let node = col.node(next);
            vec.push(node.data().unwrap().clone());
            current = node;
        }
    }

    assert_eq!(vec.len(), col.len());
    vec
}

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

fn pop_front<M>(col: &mut Col<String, M>) -> Option<String>
where
    M: MemoryPolicy<Singly<String>>,
{
    col.ends().get().cloned().map(|front_idx| {
        match col.node(&front_idx).next().get().cloned() {
            Some(new_front) => col.ends_mut().set(Some(new_front)),
            None => col.ends_mut().clear(),
        }
        col.close_and_reclaim(&front_idx)
    })
}

fn reorganize_old(col: &mut Col<String, PolicyNever>) {
    let mut first_occupied = 0;

    for vacant in 0..col.nodes().len() {
        if col.nodes()[vacant].is_closed() {
            let begin = match first_occupied > vacant {
                true => first_occupied,
                false => vacant + 1,
            };

            for occupied in begin..col.nodes().len() {
                if col.nodes()[occupied].is_active() {
                    let next_occupied = next_occupied_old(col, occupied + 1);
                    swap(col, vacant, occupied, next_occupied);

                    match next_occupied {
                        Some(next_occupied) => first_occupied = next_occupied,
                        None => return,
                    }

                    break;
                }
            }
        }
    }
}

fn next_occupied_old(col: &Col<String, PolicyNever>, start_position: usize) -> Option<usize> {
    (start_position..col.nodes().len()).find(|&i| col.nodes()[i].is_active())
}

#[test]
fn new_col() {
    let col: Col<String, PolicyNever> = SelfRefCol::new();

    assert_eq!(col.len(), 0);
    assert!(col.is_empty());
    assert_eq!(col.ends().get(), None);

    assert_eq!(forward(&col), to_str(&[]));
}

#[test]
fn push_one() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_front(&mut col, 0.to_string());

    assert_eq!(forward(&col), to_str(&[0]));
}

#[test]
fn push_front_1() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_front(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());

    assert_eq!(forward(&col), to_str(&[1, 0]));
}

#[test]
fn push_front_2() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_front(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());
    push_front(&mut col, 2.to_string());

    assert_eq!(forward(&col), to_str(&[2, 1, 0]));
}

#[test]
fn pop_empty() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    let pop = pop_front(&mut col);
    assert_eq!(pop, None);

    assert_eq!(forward(&col), to_str(&[]));
}

#[test]
fn pop_when_1() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_front(&mut col, 0.to_string());
    assert_eq!(pop_front(&mut col), Some(0.to_string()));

    assert_eq!(forward(&col), to_str(&[]));
}

#[test]
fn pop_when_3() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_front(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());
    push_front(&mut col, 2.to_string());
    assert_eq!(forward(&col), to_str(&[2, 1, 0]));

    assert_eq!(pop_front(&mut col), Some(2.to_string()));
    assert_eq!(forward(&col), to_str(&[1, 0]));

    assert_eq!(pop_front(&mut col), Some(1.to_string()));
    assert_eq!(forward(&col), to_str(&[0]));

    assert_eq!(pop_front(&mut col), Some(0.to_string()));
    assert_eq!(forward(&col), to_str(&[]));

    assert_eq!(pop_front(&mut col), None);
    assert_eq!(forward(&col), to_str(&[]));
}

#[test]
fn reorganize_col_trivial() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_front(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());
    push_front(&mut col, 2.to_string());
    push_front(&mut col, 3.to_string());
    push_front(&mut col, 4.to_string());
    push_front(&mut col, 5.to_string());

    assert_eq!(forward(&col), to_str(&[5, 4, 3, 2, 1, 0]));

    pop_front(&mut col);
    pop_front(&mut col);
    pop_front(&mut col);
    pop_front(&mut col);

    assert_eq!(forward(&col), to_str(&[1, 0]));
    assert_eq!(
        col.nodes()
            .iter()
            .map(|x| x.data().cloned())
            .collect::<Vec<_>>(),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            None,
            None,
            None,
            None
        ]
    );

    reorganize_old(&mut col);

    assert_eq!(forward(&col), to_str(&[1, 0]));
    assert_eq!(
        col.nodes()
            .iter()
            .map(|x| x.data().cloned())
            .collect::<Vec<_>>(),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            None,
            None,
            None,
            None
        ]
    );
}

#[test]
fn reorganize_col_gapped() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    let push_gap = |col: &mut Col<String, PolicyNever>| {
        push_front(col, 42.to_string());
        pop_front(col);
    };

    push_front(&mut col, 0.to_string());

    push_gap(&mut col);

    push_front(&mut col, 1.to_string());

    push_gap(&mut col);
    push_gap(&mut col);

    push_front(&mut col, 2.to_string());

    push_gap(&mut col);
    push_gap(&mut col);

    assert_eq!(forward(&col), to_str(&[2, 1, 0]));
    assert_eq!(
        col.nodes()
            .iter()
            .map(|x| x.data().cloned())
            .collect::<Vec<_>>(),
        [
            Some(0.to_string()),
            None,
            Some(1.to_string()),
            None,
            None,
            Some(2.to_string()),
            None,
            None
        ]
    );

    reorganize_old(&mut col);

    assert_eq!(forward(&col), to_str(&[2, 1, 0]));
    assert_eq!(
        col.nodes()
            .iter()
            .map(|x| x.data().cloned())
            .collect::<Vec<_>>(),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            None,
            None,
            None,
            None,
            None
        ]
    );
}

#[test]
fn reorganize_threshold() {
    let mut col: Col<String, PolicyOnThreshold<2, String>> = SelfRefCol::new();

    let push_gap = |col: &mut Col<String, PolicyOnThreshold<2, String>>| {
        push_front(col, 42.to_string());
        pop_front(col);
    };

    let nodes = |col: &Col<String, PolicyOnThreshold<2, String>>| {
        col.nodes()
            .iter()
            .map(|x| x.data().cloned())
            .collect::<Vec<_>>()
    };

    push_front(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());
    push_front(&mut col, 2.to_string());
    push_front(&mut col, 3.to_string());
    push_front(&mut col, 4.to_string());
    push_front(&mut col, 5.to_string());
    push_gap(&mut col);
    push_front(&mut col, 6.to_string());
    push_front(&mut col, 7.to_string());
    push_front(&mut col, 8.to_string());
    push_gap(&mut col);
    push_front(&mut col, 9.to_string());

    assert_eq!(forward(&col), to_str(&[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            None,
            Some(6.to_string()),
            Some(7.to_string()),
            Some(8.to_string()),
            None,
            Some(9.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[8, 7, 6, 5, 4, 3, 2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            None,
            Some(6.to_string()),
            Some(7.to_string()),
            Some(8.to_string()),
            None,
            None,
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[7, 6, 5, 4, 3, 2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            Some(7.to_string()),
            Some(6.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[6, 5, 4, 3, 2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            None,
            Some(6.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[5, 4, 3, 2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            None,
            None,
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[4, 3, 2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[3, 2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            None,
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[2, 1, 0]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[1, 0]));
    assert_eq!(nodes(&col), [Some(0.to_string()), Some(1.to_string())]);

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[0]));
    assert_eq!(nodes(&col), [Some(0.to_string())]);

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(nodes(&col), []);

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(nodes(&col), []);
}

#[test]
fn node_ref_validation_never_reclaim() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    let push_gap = |col: &mut Col<String, PolicyNever>| {
        push_front(col, 42.to_string());
        pop_front(col);
    };

    let ref0 = push_front(&mut col, 0.to_string());

    push_gap(&mut col);

    let ref1 = push_front(&mut col, 1.to_string());

    push_gap(&mut col);
    push_gap(&mut col);

    let ref2 = push_front(&mut col, 2.to_string());

    push_gap(&mut col);
    push_gap(&mut col);
    push_gap(&mut col);
    push_gap(&mut col);

    assert_eq!(forward(&col), to_str(&[2, 1, 0]));

    let refs = || [ref0, ref1, ref2];

    {
        let nodes = refs().map(|r| col.node_from_idx(&r).unwrap());
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(node.data(), Some(&i.to_string()));
        }

        let node1 = col.node(nodes[2].next().get().unwrap());
        assert_eq!(node1 as *const Node<Singly<String>>, nodes[1]);

        let node0 = col.node(nodes[1].next().get().unwrap());
        assert_eq!(node0 as *const Node<Singly<String>>, nodes[0]);

        assert!(nodes[0].next().get().is_none());

        let front_node = front(&col);
        assert_eq!(front_node as *const Node<Singly<String>>, nodes[2]);
    }

    let popped = pop_front(&mut col);
    assert_eq!(popped, Some(2.to_string()));
    assert_eq!(forward(&col), to_str(&[1, 0]));

    {
        let nodes = refs().map(|r| col.node_from_idx(&r).unwrap());

        assert_eq!(nodes[2].data(), None);
        assert!(nodes[2].next().get().is_none());

        assert_eq!(nodes[0].data(), Some(&0.to_string()));
        assert_eq!(nodes[1].data(), Some(&1.to_string()));

        let node0 = col.node(nodes[1].next().get().unwrap());
        assert_eq!(node0 as *const Node<Singly<String>>, nodes[0]);

        assert!(nodes[0].next().get().is_none());

        let front_node = front(&col);
        assert_eq!(front_node as *const Node<Singly<String>>, nodes[1]);
    }

    let popped = pop_front(&mut col);
    assert_eq!(popped, Some(1.to_string()));
    assert_eq!(forward(&col), to_str(&[0]));

    {
        let nodes = refs().map(|r| col.node_from_idx(&r).unwrap());

        assert_eq!(nodes[2].data(), None);
        assert!(nodes[2].next().get().is_none());
        assert_eq!(nodes[1].data(), None);
        assert!(nodes[1].next().get().is_none());

        assert_eq!(nodes[0].data(), Some(&0.to_string()));
        assert!(nodes[0].next().get().is_none());

        let front_node = front(&col);
        assert_eq!(front_node as *const Node<Singly<String>>, nodes[0]);
    }
}

#[test]
fn node_ref_validation_threshold_reclaim() {
    let mut col: Col<String, PolicyOnThreshold<2, String>> = SelfRefCol::new();

    let push_gap = |col: &mut Col<String, PolicyOnThreshold<2, String>>| {
        push_front(col, 42.to_string());
        pop_front(col);
    };

    let ref0 = push_front(&mut col, 0.to_string());
    let ref1 = push_front(&mut col, 1.to_string());
    let ref2 = push_front(&mut col, 2.to_string());
    let ref3 = push_front(&mut col, 3.to_string());
    push_gap(&mut col);

    let refs = || [ref0, ref1, ref2, ref3];

    assert_eq!(forward(&col), to_str(&[3, 2, 1, 0]));

    {
        let nodes = refs().map(|r| col.node_from_idx(&r).unwrap());
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(node.data(), Some(&i.to_string()));
        }

        let node2 = col.node(nodes[3].next().get().unwrap());
        assert_eq!(node2 as *const Node<Singly<String>>, nodes[2]);

        let node1 = col.node(nodes[2].next().get().unwrap());
        assert_eq!(node1 as *const Node<Singly<String>>, nodes[1]);

        let node0 = col.node(nodes[1].next().get().unwrap());
        assert_eq!(node0 as *const Node<Singly<String>>, nodes[0]);

        assert!(nodes[0].next().get().is_none());

        let front_node = front(&col);
        assert_eq!(front_node as *const Node<Singly<String>>, nodes[3]);
    }

    // reorganized but nodes are not moved; same memory state holds
    push_gap(&mut col);
    assert_eq!(forward(&col), to_str(&[3, 2, 1, 0]));
    assert_eq!(col.nodes().len(), 4);
    assert_eq!(col.len(), 4);
    {
        let nodes = refs().map(|r| col.node_from_idx(&r).unwrap());
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(node.data(), Some(&i.to_string()));
        }

        let node2 = col.node(nodes[3].next().get().unwrap());
        assert_eq!(node2 as *const Node<Singly<String>>, nodes[2]);

        let node1 = col.node(nodes[2].next().get().unwrap());
        assert_eq!(node1 as *const Node<Singly<String>>, nodes[1]);

        let node0 = col.node(nodes[1].next().get().unwrap());
        assert_eq!(node0 as *const Node<Singly<String>>, nodes[0]);

        assert!(nodes[0].next().get().is_none());

        let front_node = front(&col);
        assert_eq!(front_node as *const Node<Singly<String>>, nodes[3]);
    }

    push_gap(&mut col);
    push_front(&mut col, 4.to_string());
    push_front(&mut col, 5.to_string());
    push_front(&mut col, 6.to_string());
    assert_eq!(forward(&col), to_str(&[6, 5, 4, 3, 2, 1, 0]));
    assert_eq!(col.nodes().len(), 8);
    assert_eq!(col.len(), 7);

    push_gap(&mut col);
    assert_eq!(forward(&col), to_str(&[6, 5, 4, 3, 2, 1, 0]));
    assert_eq!(col.nodes().len(), 9);
    assert_eq!(col.len(), 7);
    {
        let nodes = refs().map(|r| col.node_from_idx(&r).unwrap());
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(node.data(), Some(&i.to_string()));
        }

        let node2 = col.node(nodes[3].next().get().unwrap());
        assert_eq!(node2 as *const Node<Singly<String>>, nodes[2]);

        let node1 = col.node(nodes[2].next().get().unwrap());
        assert_eq!(node1 as *const Node<Singly<String>>, nodes[1]);

        let node0 = col.node(nodes[1].next().get().unwrap());
        assert_eq!(node0 as *const Node<Singly<String>>, nodes[0]);

        assert!(nodes[0].next().get().is_none());

        let front_node = front(&col);
        assert_ne!(front_node as *const Node<Singly<String>>, nodes[3]);
    }

    let popped = pop_front(&mut col);
    assert_eq!(popped, Some(6.to_string()));
    assert_eq!(forward(&col), to_str(&[5, 4, 3, 2, 1, 0]));
    assert_eq!(col.nodes().len(), 6);
    assert_eq!(col.len(), 6); // reorganized and moved => state changed

    {
        let nodes = refs().map(|r| col.node_from_idx(&r));
        let all_invalid = nodes.iter().all(|x| x.is_none());
        assert!(all_invalid);
    }
}
