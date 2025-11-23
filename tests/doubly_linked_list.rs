use orx_iterable::Collection;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::*;
use orx_split_vec::{Recursive, SplitVec};
use std::marker::PhantomData;

struct Doubly<T>(PhantomData<T>);

type PolicyNever = MemoryReclaimNever;
type PolicyOnThreshold<const D: usize, T> =
    MemoryReclaimOnThreshold<D, Doubly<T>, OnThresholdReclaimer>;

#[derive(Clone, Default)]
pub struct OnThresholdReclaimer;
impl<T> MemoryReclaimer<Doubly<T>> for OnThresholdReclaimer {
    fn reclaim_nodes<P>(col: &mut CoreCol<Doubly<T>, P>) -> bool
    where
        P: PinnedVec<Node<Doubly<T>>>,
    {
        let mut any_swapped = false;
        let mut right_bound = col.nodes().len();

        for vacant in 0..col.nodes().len() {
            if col.nodes()[vacant].is_closed() {
                for occupied in ((vacant + 1)..right_bound).rev() {
                    if col.nodes()[occupied].is_active() {
                        right_bound = occupied;
                        swap(col, vacant, occupied);
                        any_swapped = true;
                        break;
                    }
                }
            }
        }
        any_swapped
    }
}

fn swap<P, T>(col: &mut CoreCol<Doubly<T>, P>, vacant: usize, occupied: usize)
where
    P: PinnedVec<Node<Doubly<T>>>,
{
    let new_idx = col.node_ptr_at_pos(vacant);
    let old_idx = col.node_ptr_at_pos(occupied);

    if let Some(prev) = col.nodes()[occupied].prev().get() {
        col.node_mut(prev).next_mut().set(Some(new_idx));
    }

    if let Some(next) = col.nodes()[occupied].next().get() {
        col.node_mut(next).prev_mut().set(Some(new_idx));
    }

    col.move_node(vacant, occupied);

    if old_idx == col.ends().get(0).expect("nonempty list") {
        col.ends_mut().set(0, Some(new_idx));
    }

    if old_idx == col.ends().get(1).expect("nonempty list") {
        col.ends_mut().set(1, Some(new_idx));
    }
}

impl<T> Variant for Doubly<T> {
    type Item = T;

    type Prev = RefsSingle<Self>;

    type Next = RefsSingle<Self>;

    type Ends = RefsArray<2, Self>;
}

type Col<T, M> = SelfRefCol<Doubly<T>, M, SplitVec<Node<Doubly<T>>, Recursive>>;

fn to_str(numbers: &[usize]) -> Vec<String> {
    numbers.iter().map(|x| x.to_string()).collect()
}

fn forward<M>(col: &Col<String, M>) -> Vec<String>
where
    M: MemoryPolicy<Doubly<String>>,
{
    let mut vec = vec![];

    if !col.is_empty() {
        let [front, _] = front_back(col);
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

fn backward<M>(col: &Col<String, M>) -> Vec<String>
where
    M: MemoryPolicy<Doubly<String>>,
{
    let mut vec = vec![];

    if !col.is_empty() {
        let [_, back] = front_back(col);
        vec.push(back.data().unwrap().clone());

        let mut current = back;

        while let Some(prev) = current.prev().get() {
            let node = col.node(prev);
            vec.push(node.data().unwrap().clone());
            current = node;
        }
    }

    assert_eq!(vec.len(), col.len());
    vec
}

fn front<M>(col: &Col<String, M>) -> Option<NodePtr<Doubly<String>>>
where
    M: MemoryPolicy<Doubly<String>>,
{
    col.ends().get(0)
}

fn back<M>(col: &Col<String, M>) -> Option<NodePtr<Doubly<String>>>
where
    M: MemoryPolicy<Doubly<String>>,
{
    col.ends().get(1)
}

fn front_back<M>(col: &Col<String, M>) -> [&Node<Doubly<String>>; 2]
where
    M: MemoryPolicy<Doubly<String>>,
{
    [
        col.node(col.ends().get(0).unwrap()),
        col.node(col.ends().get(1).unwrap()),
    ]
}

fn get_at<M>(col: &Col<String, M>, at: usize) -> Option<NodePtr<Doubly<String>>>
where
    M: MemoryPolicy<Doubly<String>>,
{
    let [len, half_len] = [col.len(), col.len() / 2];

    match at {
        x if x < half_len => {
            let mut current = front(col).expect("non-empty list");
            for _ in 0..at {
                current = col.node(current).next().get().expect("must exist");
            }
            Some(current)
        }
        x if x < len => {
            let mut current = back(col).expect("non-empty list");
            let num_jumps = len - at - 1;
            for _ in 0..num_jumps {
                current = col.node(current).prev().get().expect("must exist");
            }
            Some(current)
        }
        _ => None,
    }
}

fn push_first<M>(col: &mut Col<String, M>, value: String) -> NodeIdx<Doubly<String>>
where
    M: MemoryPolicy<Doubly<String>>,
{
    let ptr = col.push(value);
    col.ends_mut().set(0, Some(ptr));
    col.ends_mut().set(1, Some(ptr));
    NodeIdx::new(col.memory_state(), ptr)
}

fn push_front<M>(col: &mut Col<String, M>, value: String)
where
    M: MemoryPolicy<Doubly<String>>,
{
    let idx = col.push(value);
    let old_front = col.ends().get(0).unwrap();

    col.node_mut(idx).next_mut().set(Some(old_front));
    col.node_mut(old_front).prev_mut().set(Some(idx));
    col.ends_mut().set(0, Some(idx));
}

fn push_back<M>(col: &mut Col<String, M>, value: String) -> NodeIdx<Doubly<String>>
where
    M: MemoryPolicy<Doubly<String>>,
{
    let ptr = col.push(value);
    let old_back = col.ends().get(1).unwrap();

    col.node_mut(ptr).prev_mut().set(Some(old_back));
    col.node_mut(old_back).next_mut().set(Some(ptr));
    col.ends_mut().set(1, Some(ptr));
    NodeIdx::new(col.memory_state(), ptr)
}

fn pop_front<M>(col: &mut Col<String, M>) -> Option<String>
where
    M: MemoryPolicy<Doubly<String>>,
{
    col.ends().get(0).map(|front_idx| {
        match col.node(front_idx).next().get() {
            Some(new_front) => {
                col.node_mut(new_front).prev_mut().clear();
                col.ends_mut().set(0, Some(new_front));
            }
            None => col.ends_mut().clear(),
        }

        col.close_and_reclaim(front_idx)
    })
}

fn pop_back<M>(col: &mut Col<String, M>) -> Option<String>
where
    M: MemoryPolicy<Doubly<String>>,
{
    col.ends().get(1).map(|back_idx| {
        match col.node(back_idx).prev().get() {
            Some(new_back) => {
                col.node_mut(new_back).next_mut().clear();
                col.ends_mut().set(1, Some(new_back));
            }
            None => col.ends_mut().clear(),
        }
        col.close_and_reclaim(back_idx)
    })
}

fn remove_at<M>(col: &mut Col<String, M>, at: usize) -> Option<String>
where
    M: MemoryPolicy<Doubly<String>>,
{
    match at {
        0 => pop_front(col),
        x if x < col.len() => match x == col.len() - 1 {
            false => {
                let node_idx = get_at(col, at).expect("in bounds");

                let [prev, next] = {
                    let node = col.node(node_idx);
                    [node.prev().get(), node.next().get()]
                };

                match prev {
                    Some(prev) => col.node_mut(prev).next_mut().set(next),
                    None => col.ends_mut().set(0, next),
                }

                match next {
                    Some(next) => col.node_mut(next).prev_mut().set(prev),
                    None => col.ends_mut().set(1, prev),
                }

                Some(col.close_and_reclaim(node_idx))
            }
            true => pop_back(col),
        },
        _ => None,
    }
}

#[test]
fn new_col() {
    let col: Col<String, PolicyNever> = SelfRefCol::new();

    assert_eq!(col.len(), 0);
    assert!(col.is_empty());
    assert_eq!(col.ends().get(0), None);
    assert_eq!(col.ends().get(1), None);

    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));
}

#[test]
fn push_one() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());

    assert_eq!(col.len(), 1);
    assert!(!col.is_empty());

    assert_eq!(forward(&col), to_str(&[0]));
    assert_eq!(backward(&col), to_str(&[0]));
}

#[test]
fn push_front_1() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());

    assert_eq!(forward(&col), to_str(&[1, 0]));
    assert_eq!(backward(&col), to_str(&[0, 1]));
}

#[test]
fn push_back_1() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    push_back(&mut col, 1.to_string());

    assert_eq!(forward(&col), to_str(&[0, 1]));
    assert_eq!(backward(&col), to_str(&[1, 0]));
}

#[test]
fn push_front_2() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());
    push_front(&mut col, 2.to_string());

    assert_eq!(forward(&col), to_str(&[2, 1, 0]));
    assert_eq!(backward(&col), to_str(&[0, 1, 2]));
}

#[test]
fn push_back_2() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    push_back(&mut col, 1.to_string());
    push_back(&mut col, 2.to_string());

    assert_eq!(forward(&col), to_str(&[0, 1, 2]));
    assert_eq!(backward(&col), to_str(&[2, 1, 0]));
}

#[test]
fn push_front_back() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    push_back(&mut col, 1.to_string());
    push_front(&mut col, 2.to_string());

    assert_eq!(forward(&col), to_str(&[2, 0, 1]));
    assert_eq!(backward(&col), to_str(&[1, 0, 2]));
}

#[test]
fn pop_front_empty() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    let pop = pop_front(&mut col);
    assert_eq!(pop, None);

    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));
}

#[test]
fn pop_front_when_1() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    assert_eq!(pop_front(&mut col), Some(0.to_string()));

    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));
}

#[test]
fn pop_front_when_3() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    push_front(&mut col, 1.to_string());
    push_front(&mut col, 2.to_string());

    assert_eq!(pop_front(&mut col), Some(2.to_string()));
    assert_eq!(forward(&col), to_str(&[1, 0]));
    assert_eq!(backward(&col), to_str(&[0, 1]));

    assert_eq!(pop_front(&mut col), Some(1.to_string()));
    assert_eq!(forward(&col), to_str(&[0]));
    assert_eq!(backward(&col), to_str(&[0]));

    assert_eq!(pop_front(&mut col), Some(0.to_string()));
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));

    assert_eq!(pop_front(&mut col), None);
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));
}

#[test]
fn pop_back_empty() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    let pop = pop_back(&mut col);
    assert_eq!(pop, None);

    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));
}

#[test]
fn pop_back_when_1() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    assert_eq!(pop_back(&mut col), Some(0.to_string()));

    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));
}

#[test]
fn pop_back_when_3() {
    let mut col: Col<String, PolicyNever> = SelfRefCol::new();

    push_first(&mut col, 0.to_string());
    push_back(&mut col, 1.to_string());
    push_back(&mut col, 2.to_string());
    assert_eq!(forward(&col), to_str(&[0, 1, 2]));
    assert_eq!(backward(&col), to_str(&[2, 1, 0]));

    assert_eq!(pop_back(&mut col), Some(2.to_string()));
    assert_eq!(forward(&col), to_str(&[0, 1]));
    assert_eq!(backward(&col), to_str(&[1, 0]));

    assert_eq!(pop_back(&mut col), Some(1.to_string()));
    assert_eq!(forward(&col), to_str(&[0]));
    assert_eq!(backward(&col), to_str(&[0]));

    assert_eq!(pop_back(&mut col), Some(0.to_string()));
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));

    assert_eq!(pop_back(&mut col), None);
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(backward(&col), to_str(&[]));
}

#[test]
fn reorganize_threshold() {
    let mut col: Col<String, PolicyOnThreshold<2, String>> = SelfRefCol::new();

    let nodes = |col: &Col<String, PolicyOnThreshold<2, String>>| {
        col.nodes()
            .iter()
            .map(|x| x.data().cloned())
            .collect::<Vec<_>>()
    };

    push_first(&mut col, 0.to_string());
    push_back(&mut col, 1.to_string());
    push_back(&mut col, 2.to_string());
    push_back(&mut col, 3.to_string());
    push_back(&mut col, 4.to_string());
    push_back(&mut col, 5.to_string());
    push_back(&mut col, 6.to_string());
    push_back(&mut col, 7.to_string());
    push_back(&mut col, 8.to_string());

    assert_eq!(forward(&col), to_str(&[0, 1, 2, 3, 4, 5, 6, 7, 8]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            Some(6.to_string()),
            Some(7.to_string()),
            Some(8.to_string()),
        ]
    );

    pop_back(&mut col);
    assert_eq!(forward(&col), to_str(&[0, 1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            Some(6.to_string()),
            Some(7.to_string()),
            None
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[1, 2, 3, 4, 5, 6, 7]));
    assert_eq!(
        nodes(&col),
        [
            None,
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            Some(6.to_string()),
            Some(7.to_string()),
            None
        ]
    );

    pop_back(&mut col);
    assert_eq!(forward(&col), to_str(&[1, 2, 3, 4, 5, 6]));
    assert_eq!(
        nodes(&col),
        [
            Some(6.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[2, 3, 4, 5, 6]));
    assert_eq!(
        nodes(&col),
        [
            Some(6.to_string()),
            None,
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[3, 4, 5, 6]));
    assert_eq!(
        nodes(&col),
        [
            Some(6.to_string()),
            Some(5.to_string()),
            Some(4.to_string()),
            Some(3.to_string()),
        ]
    );

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[4, 5, 6]));
    assert_eq!(
        nodes(&col),
        [
            Some(6.to_string()),
            Some(5.to_string()),
            Some(4.to_string()),
            None,
        ]
    );

    pop_back(&mut col);
    assert_eq!(forward(&col), to_str(&[4, 5]));
    assert_eq!(nodes(&col), [Some(4.to_string()), Some(5.to_string())]);

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[5]));
    assert_eq!(nodes(&col), [Some(5.to_string())]);

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(nodes(&col), []);

    pop_front(&mut col);
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(nodes(&col), []);
}

#[test]
fn remove_at_test() {
    let mut col: Col<String, PolicyOnThreshold<2, String>> = SelfRefCol::new();

    let nodes = |col: &Col<String, PolicyOnThreshold<2, String>>| {
        col.nodes()
            .iter()
            .map(|x| x.data().cloned())
            .collect::<Vec<_>>()
    };

    push_first(&mut col, 0.to_string());
    push_back(&mut col, 1.to_string());
    push_back(&mut col, 2.to_string());
    push_back(&mut col, 3.to_string());
    push_back(&mut col, 4.to_string());
    push_back(&mut col, 5.to_string());
    push_back(&mut col, 6.to_string());
    push_back(&mut col, 7.to_string());
    push_back(&mut col, 8.to_string());

    assert_eq!(forward(&col), to_str(&[0, 1, 2, 3, 4, 5, 6, 7, 8]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            Some(3.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
            Some(6.to_string()),
            Some(7.to_string()),
            Some(8.to_string()),
        ]
    );

    let removed = remove_at(&mut col, 3);
    assert_eq!(removed, Some(3.to_string()));
    assert_eq!(forward(&col), to_str(&[0, 1, 2, 4, 5, 6, 7, 8]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            None,
            Some(4.to_string()),
            Some(5.to_string()),
            Some(6.to_string()),
            Some(7.to_string()),
            Some(8.to_string()),
        ]
    );

    let removed = remove_at(&mut col, 6);
    assert_eq!(removed, Some(7.to_string()));
    assert_eq!(forward(&col), to_str(&[0, 1, 2, 4, 5, 6, 8]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(1.to_string()),
            Some(2.to_string()),
            None,
            Some(4.to_string()),
            Some(5.to_string()),
            Some(6.to_string()),
            None,
            Some(8.to_string()),
        ]
    );

    let removed = remove_at(&mut col, 1);
    assert_eq!(removed, Some(1.to_string()));
    assert_eq!(forward(&col), to_str(&[0, 2, 4, 5, 6, 8]));
    assert_eq!(
        nodes(&col),
        [
            Some(0.to_string()),
            Some(8.to_string()),
            Some(2.to_string()),
            Some(6.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
        ]
    );

    let removed = remove_at(&mut col, 0);
    assert_eq!(removed, Some(0.to_string()));
    assert_eq!(forward(&col), to_str(&[2, 4, 5, 6, 8]));
    assert_eq!(
        nodes(&col),
        [
            None,
            Some(8.to_string()),
            Some(2.to_string()),
            Some(6.to_string()),
            Some(4.to_string()),
            Some(5.to_string()),
        ]
    );

    let removed = remove_at(&mut col, 4);
    assert_eq!(removed, Some(8.to_string()));
    assert_eq!(forward(&col), to_str(&[2, 4, 5, 6]));
    assert_eq!(
        nodes(&col),
        [
            Some(5.to_string()),
            Some(4.to_string()),
            Some(2.to_string()),
            Some(6.to_string()),
        ]
    );

    let removed = remove_at(&mut col, 2);
    assert_eq!(removed, Some(5.to_string()));
    assert_eq!(forward(&col), to_str(&[2, 4, 6]));
    assert_eq!(
        nodes(&col),
        [
            None,
            Some(4.to_string()),
            Some(2.to_string()),
            Some(6.to_string()),
        ]
    );

    let removed = remove_at(&mut col, 1);
    assert_eq!(removed, Some(4.to_string()));
    assert_eq!(forward(&col), to_str(&[2, 6]));
    assert_eq!(nodes(&col), [Some(6.to_string()), Some(2.to_string())]);

    let removed = remove_at(&mut col, 3);
    assert_eq!(removed, None);
    assert_eq!(forward(&col), to_str(&[2, 6]));
    assert_eq!(nodes(&col), [Some(6.to_string()), Some(2.to_string())]);

    let removed = remove_at(&mut col, 1);
    assert_eq!(removed, Some(6.to_string()));
    assert_eq!(forward(&col), to_str(&[2]));
    assert_eq!(nodes(&col), [Some(2.to_string())]);

    let removed = remove_at(&mut col, 0);
    assert_eq!(removed, Some(2.to_string()));
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(nodes(&col), []);

    let removed = remove_at(&mut col, 0);
    assert_eq!(removed, None);
    assert_eq!(forward(&col), to_str(&[]));
    assert_eq!(nodes(&col), []);
}

#[test]
fn send_node_idx() {
    let mut col: Col<String, PolicyOnThreshold<2, String>> = SelfRefCol::new();

    let indices = vec![
        push_first(&mut col, 0.to_string()),
        push_back(&mut col, 1.to_string()),
        push_back(&mut col, 2.to_string()),
        push_back(&mut col, 3.to_string()),
        push_back(&mut col, 4.to_string()),
        push_back(&mut col, 5.to_string()),
        push_back(&mut col, 6.to_string()),
        push_back(&mut col, 7.to_string()),
        push_back(&mut col, 8.to_string()),
    ];

    let state = col.memory_state();

    std::thread::scope(|s| {
        let indices = indices.as_slice();
        let col = &col;
        for (i, idx) in indices.iter().enumerate().take(9) {
            s.spawn(move || {
                assert!(idx.is_in_state(state));
                assert!(idx.is_valid_for(col));
                let value = idx.node(col).and_then(|n| n.data().cloned());
                assert_eq!(value, Some(i.to_string()));
            });
        }
    })
}
