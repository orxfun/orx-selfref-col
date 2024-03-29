//! # orx-selfref-col
//!
//! [![orx-selfref-col crate](https://img.shields.io/crates/v/orx-selfref-col.svg)](https://crates.io/crates/orx-selfref-col)
//! [![orx-selfref-col documentation](https://docs.rs/orx-selfref-col/badge.svg)](https://docs.rs/orx-selfref-col)
//!
//! `SelfRefCol` is a core data structure to conveniently build safe and efficient self referential collections, such as linked lists and trees.
//!
//! ## Features
//!
//! ### Safe
//!
//! It is tricky to implement safe self referential collections.
//! Rust's safety features are conservative in this case and encourages using wide smart pointers which lead to poor cache locality.
//! To avoid this, one can use indices which is neither nice or safe.
//!
//! `SelfRefCol` constructs its safety guarantees around the following:
//! * memory locations of elements of the collection will never change unless explicitly changed due to the pinned elements guarantees of the underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec),
//! * all references will be ensured to be among elements of the same collection.
//!
//! In other words,
//! * by keeping the references valid,
//! * preventing to bring in external references, and
//! * preventing to leak out references,
//!
//! it is safe to build the self referential collection with regular `&` references.
//!
//! [orx-linked-list](https://crates.io/crates/orx-linked-list) crate defines both singly and doubly linked lists based on `SelfRefCol`:
//! * without any pointer dereferencing,
//! * without any access by indices, and
//! * with only a couple unsafe calls.
//!
//! Note that the [`std::collections::linked_list::LinkedList`](https://doc.rust-lang.org/src/alloc/collections/linked_list.rs.html) implementation contains more than 60 `unsafe` code blocks.
//!
//! ### Convenient
//!
//! `SelfRefCol`, specializing in building self referential collections, provides (only) the required methods to builds such collections.
//! This allows to avoid the need for general low level types or methods which are alien to the structure to be defined, such as `NonNull`, `Box::from_raw_in`, `mem::replace`, etc.
//! Instead, the implementation can be concise, expressive and close to the pseudocode of the method.
//!
//! This might be more clear with an example.
//! Consider the following `push_back` method of the doubly linked list defined in [orx-linked-list](https://crates.io/crates/orx-linked-list) using `SelfRefCol`:
//!
//! ```rust ignore
//! pub fn push_back(&mut self, value: T) {
//!     self.col
//!         .move_mutate(value, |x, value| match x.ends().back() {
//!             Some(prior_back) => {
//!                 let new_back = x.push_get_ref(value);
//!                 new_back.set_prev(&x, prior_back);
//!                 prior_back.set_next(&x, new_back);
//!                 x.set_ends([x.ends().front(), Some(new_back)]);
//!             }
//!             None => {
//!                 let node = x.push_get_ref(value);
//!                 x.set_ends([Some(node), Some(node)]);
//!             }
//!         });
//! }
//! ```
//!
//! Note that all words are relevant to a linked list, except for `x`, which is an internal mutability key which use used inside a non-capturing lambda to prevent reference leaks.
//!
//! ### Efficient
//!
//! The elements of the `SelfRefCol` are internally stored in a [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) implementation, which is crucial for the correctness and safety of the references.
//! Furthermore, in this way, elements of the collection are stored close to each other rather than being in arbitrary locations in memory.
//! This provides better cache locality when compared to such collections where elements are stored by arbitrary heap allocations.
//!
//! With the current benchmarks, [orx-linked-list](https://crates.io/crates/orx-linked-list) implementation using `SelfRefCol` is at least three times faster than the `std::collections::linked_list::LinkedList`.
//!
//! ## Variants
//!
//! Note that this core structure is capable of representing a wide range of self referential collections, where the variant is conveniently defined by expressive trait type definitions.
//!
//! The represented collections have the following features:
//! * Relations are represented by regular `&` references avoiding the need to use smart pointers such as `Box`, `Rc`, `Arc`, etc.
//! * The collection makes sure that these references are set only among elements of the collections.
//! In other words, no external references are allowed, or references of elements of the collection cannot leak out.
//! This constructs the safety guarantees.
//! * The elements of the collection are internally stored in a `PinnedVec` implementation, which is crucial for the correctness of the references.
//! Furthermore, in this way, elements of the collection are stored close to each other rather than being in arbitrary locations in memory.
//! This provides better cache locality when compared to such collections where elements are stored by arbitrary heap allocations.
//!
//! The collection is defined by the following generic arguments:
//! * `T`: type of the elements stored in the collection.
//! * `V`: type of the `Variant` defining the structure of the collection with the following:
//!   * `V::Storage`: defines how the elements of `T` will be stored:
//!     * `NodeDataLazyClose`: elements are stored as `Option<T>` allowing lazy node closure or element removal;
//!     * `NodeDataEagerClose`: elements are stored directly as `T`.
//!   * `V::Prev`: defines how references to previous elements will be stored.
//!     * `NodeRefNone`: there is no previous reference of elements.
//!     * `NodeRefSingle`: there is either one or no previous reference of elements, stored as `Option<&Node>`.
//!     * `NodeRefsArray`: there are multiple possible previous references up to a constant number `N`, stored as `[Option<&Node>; N]`.
//!     * `NodeRefsVec`: there are multiple possible previous references, stored as `Vec<&Node>`.
//!   * `V::Next`: defines how references to next elements will be stored:
//!     * Similarly, represented as either one of `NodeRefNone` or `NodeRefSingle` or `NodeRefsArray` or `NodeRefsVec`.
//!   * `V::Ends`: defines how references to ends of the collection will be stored:
//!     * Similarly, represented as either one of `NodeRefNone` or `NodeRefSingle` or `NodeRefsArray` or `NodeRefsVec`.
//!   * `V::MemoryReclaim`: defines how memory of closed nodes will be reclaimed:
//!     * `MemoryReclaimNever` will never claim closed nodes.
//!     * `MemoryReclaimOnThreshold<D>` will claim memory of closed nodes whenever the ratio of closed nodes exceeds one over `2^D`.
//!
//! ### Example
//!
//! Consider the following four structs implementing `Variant` to define four different self referential collections.
//! Note that the definitions are expressive and concise leading to efficient implementations.
//!
//! ```rust
//! use orx_selfref_col::*;
//!
//! #[derive(Clone, Copy)]
//! struct SinglyListVariant;
//!
//! impl<'a, T: 'a> Variant<'a, T> for SinglyListVariant {
//!     type Storage = NodeDataLazyClose<T>; // lazy close
//!     type MemoryReclaim = MemoryReclaimOnThreshold<2>; // closed nodes will be reclaimed when utilization drops below 75%
//!     type Prev = NodeRefNone; // previous nodes are not stored
//!     type Next = NodeRefSingle<'a, Self, T>; // there is only one next node, if any
//!     type Ends = NodeRefSingle<'a, Self, T>; // there is only one end, namely the front of the list
//! }
//!
//! #[derive(Clone, Copy)]
//! struct DoublyListVariant;
//!
//! impl<'a, T: 'a> Variant<'a, T> for DoublyListVariant {
//!     type Storage = NodeDataLazyClose<T>; // lazy close
//!     type MemoryReclaim = MemoryReclaimOnThreshold<3>; // closed nodes will be reclaimed when utilization drops below 87.5%
//!     type Prev = NodeRefSingle<'a, Self, T>; // there is only one previous node, if any
//!     type Next = NodeRefSingle<'a, Self, T>; // there is only one next node, if any
//!     type Ends = NodeRefsArray<'a, 2, Self, T>; // there are two ends, namely the front and back of the list
//! }
//!
//! #[derive(Clone, Copy)]
//! struct BinaryTreeVariant;
//!
//! impl<'a, T: 'a> Variant<'a, T> for BinaryTreeVariant {
//!     type Storage = NodeDataLazyClose<T>; // lazy close
//!     type MemoryReclaim = MemoryReclaimOnThreshold<1>; // closed nodes will be reclaimed when utilization drops below 50%
//!     type Prev = NodeRefSingle<'a, Self, T>; // there is only one previous node, namely parent node, if any
//!     type Next = NodeRefsArray<'a, 2, Self, T>; // there are 0, 1 or 2 next or children nodes
//!     type Ends = NodeRefSingle<'a, Self, T>; // there is only one end, namely the root of the tree
//! }
//!
//! #[derive(Clone, Copy)]
//! struct DynamicTreeVariant;
//!
//! impl<'a, T: 'a> Variant<'a, T> for DynamicTreeVariant {
//!     type Storage = NodeDataLazyClose<T>; // lazy close
//!     type MemoryReclaim = MemoryReclaimNever; // closed nodes will be left as holes
//!     type Prev = NodeRefSingle<'a, Self, T>; // there is only one previous node, namely parent node, if any
//!     type Next = NodeRefsVec<'a, Self, T>; // there might be any number of next nodes, namely children nodes
//!     type Ends = NodeRefSingle<'a, Self, T>; // there is only one end, namely the root of the tree
//! }
//! ```
//!
//! ## `NodeIndex`
//!
//! `NodeIndex` belongs in the intersection of the two features efficient and safe.
//!
//! A `NodeIndex` is a struct holding a reference to an element of the collection. It provides constant time access to the element. Furthermore, a node index can be stored independently of the collection, elsewhere.
//!
//! In this sense `NodeIndex` to a `SelfRefCol` is analogous to `usize` to standard `Vec`.
//!
//! However, it puts a special emphasis on safety and correctness. The following invalid uses cannot happen with `NodeIndex` and `SelfRefCol`.
//!
//! ### Cannot use a `NodeIndex` on a wrong `SelfRefCol`
//!
//! This issue can be observed with `usize` but not with `NodeIndex`.
//!
//! ```rust ignore
//! use orx_selfref_col::*;
//!
//! let mut col1 = SelfRefCol::<Var, _>::new();
//! let a = col1.mutate_take('a', |x, a| x.push_get_ref(a).index(&x));
//!
//! let col2 = SelfRefCol::<Var, _>::new();
//!
//! assert!(!a.is_valid_for_collection(&col2)); // we cannot dereference 'a' with 'col2'!
//! ```
//!
//! ### Cannot use a `NodeIndex` after the corresponding element is removed
//!
//! This issue can be observed with `usize` but not with `NodeIndex`.
//!
//! ```rust ignore
//! let mut col = SelfRefCol::<Var, _>::new();
//! let [a, b, c, d, e, f, g] = col
//!     .mutate_take(['a', 'b', 'c', 'd', 'e', 'f', 'g'], |x, values| {
//!         values.map(|val| x.push_get_ref(val).index(&x))
//!     });
//!
//! let removed_b = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // does not trigger reclaim yet
//!
//! assert!(!b.is_valid_for_collection(&col)); // we cannot dereference 'b' as it is removed.
//! ```
//!
//! ### Cannot use a `NodeIndex` after a reorganization of the elements
//!
//! This is a specific to `SelfRefCol` with lazy node closure, only when `MemoryReclaimOnThreshold` policy is used. It never happens for `MemoryReclaimNever` policy.
//!
//! Once the node utilization drops down a determined value, the collection reorganizes its elements and reclaims the memory of the closed nodes. This invalidates all indices created beforehand, preventing any wrong access.
//!
//! This is analogous to removing the first element of a vector while our index is pointing to the 3rd element. After the removal, the elements of the vector are reorganized, and our index is now pointing to a wrong element. `NodeIndex` does not allow such an access.
//!
//! ```rust ignore
//! let mut col = SelfRefCol::<Var, _>::new();
//! let [a, b, c] = col.mutate_take(['a', 'b', 'c'], |x, values| {
//!     values.map(|val| x.push_get_ref(val).index(&x))
//! });
//!
//! let removed_b = col.mutate_take(b, |x, b| b.as_ref(&x).close_node_take_data(&x)); // triggers reclaim, elements reorganized
//!
//! assert!(!a.is_valid_for_collection(&col)); // all prior indices are invalidated!
//! assert!(!b.is_valid_for_collection(&col));
//! assert!(!c.is_valid_for_collection(&col));
//! ```
//!
//! ## Crates using `SelfRefCol`
//!
//! The following crates use `SelfRefCol` to conveniently build the corresponding data structure:
//! * [orx-linked-list](https://crates.io/crates/orx-linked-list): implements singly and doubly linked lists.
//!
//! ## License
//!
//! This library is licensed under MIT license. See LICENSE for details.

#![warn(
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::missing_panics_doc,
    clippy::todo
)]

mod common_traits;
mod data;
mod memory_reclaim;
mod nodes;
mod references;
mod selfref_col;
mod selfref_col_mut;
mod selfref_col_visit;
mod variants;

pub use data::{
    eager_close::NodeDataEagerClose, lazy_close::NodeDataLazyClose, node_data::NodeData,
};
pub use nodes::{can_leak::CanLeak, index::NodeIndex, index_error::NodeIndexError, node::Node};
pub use references::{
    array::NodeRefsArray, node_refs::NodeRefs, none::NodeRefNone, single::NodeRefSingle,
    vec::NodeRefsVec,
};
pub use selfref_col::SelfRefCol;
pub use selfref_col_mut::SelfRefColMut;
pub use selfref_col_visit::SelfRefColVisit;
pub use variants::memory_reclaim::{
    MemoryReclaimNever, MemoryReclaimOnThreshold, MemoryReclaimPolicy, Reclaim,
};
pub use variants::variant::Variant;
