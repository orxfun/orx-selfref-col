# orx-selfref-col

[![orx-selfref-col crate](https://img.shields.io/crates/v/orx-selfref-col.svg)](https://crates.io/crates/orx-selfref-col)
[![orx-selfref-col documentation](https://docs.rs/orx-selfref-col/badge.svg)](https://docs.rs/orx-selfref-col)

`SelfRefCol` is a core data structure to conveniently build safe and efficient self referential collections, such as linked lists and trees.

## Features

### Safe

It is tricky to implement safe self referential collections, `SelfRefCol` provides a safe and convenient api to enable them with the following approach:
* memory locations of elements of the collection will never change unless explicitly changed due to the pinned elements guarantees of the underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec),
* all references will be ensured to be among elements of the same collection.

### Efficient

The elements of the `SelfRefCol` are internally stored in a [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) implementation, which is crucial for the correctness and safety of the references.
Furthermore, in this way, elements of the collection are stored close to each other rather than being in arbitrary locations in memory in order to improve cache locality.

## Variants

Note that this core structure is capable of representing a wide range of self referential collections, where the variant is conveniently defined by expressive trait type definitions.

* `V: Variant`: defines the structure of the collection with the following:
  * `V::Item`: is the type of the items or elements.
  * `V::Prev`: defines how references to previous elements will be stored.
    * `RefsNone`: there is no previous reference of elements.
    * `RefsSingle`: there is either one or no previous reference of elements, stored as `Option<&Node>`.
    * `RefsArray`: there are multiple possible previous references up to a constant number `N`, stored as `[Option<&Node>; N]`.
    * `RefsVec`: there are multiple possible previous references, stored as `Vec<&Node>`.
  * `V::Next`: defines how references to next elements will be stored:
    * Similarly, represented as either one of `RefsNone` or `RefsSingle` or `RefsArray` or `RefsVec`.
  * `V::Ends`: defines how references to ends of the collection will be stored:
    * Similarly, represented as either one of `RefsNone` or `RefsSingle` or `RefsArray` or `RefsVec`.
* `M: MemoryReclaimPolicy`: defines how memory of closed nodes will be reclaimed:
    * `MemoryReclaimNever` will never claim closed nodes.
    * `MemoryReclaimOnThreshold<D>` will claim memory of closed nodes whenever the ratio of closed nodes exceeds one over `2^D`.

### Example

Consider the following four structs implementing `Variant` to define four different self referential collections.
Note that the definitions are expressive and concise leading to efficient implementations.

```rust
use orx_selfref_col::*;
use core::marker::PhantomData;

pub struct Singly<T> {
    p: PhantomData<T>,
}
impl<T> Variant for Singly<T> {
    type Item = T;
    type Prev = RefsNone;
    type Next = RefsSingle<Self>;
    type Ends = RefsSingle<Self>; // front
}

pub struct Doubly<T> {
    p: PhantomData<T>,
}
impl<T> Variant for Doubly<T> {
    type Item = T;
    type Prev = RefsSingle<Self>;
    type Next = RefsSingle<Self>;
    type Ends = RefsArray<2, Self>; // front & back
}

pub struct BinaryTree<T> {
    p: PhantomData<T>,
}
impl<T> Variant for BinaryTree<T> {
    type Item = T;
    type Prev = RefsSingle<Self>;   // parent
    type Next = RefsArray<2, Self>; // 2 children
    type Ends = RefsSingle<Self>;   // root
}

pub struct DynamicTree<T> {
    p: PhantomData<T>,
}
impl<T> Variant for DynamicTree<T> {
    type Item = T;
    type Prev = RefsSingle<Self>;   // parent
    type Next = RefsVec<Self>;      // n children
    type Ends = RefsSingle<Self>;   // root
}
```

## `NodeIndex`

`NodeIndex` belongs in the intersection of the two features efficient and safe.

A `NodeIndex` is a struct holding a reference to an element of the collection. It provides constant time access to the element. Furthermore, a node index can be stored independently of the collection, elsewhere.

In this sense `NodeIndex` to a `SelfRefCol` is sort of analogous to `usize` to standard `Vec`.

However, it puts a special emphasis on safety and correctness. The following invalid uses cannot happen with `NodeIndex` and `SelfRefCol`. The following safety guarantees are provided by the self referential collection:

* cannot use a `NodeIndex` on a wrong `SelfRefCol`
* cannot use a `NodeIndex` after the corresponding element is removed
* cannot use a `NodeIndex` after a reorganization of the elements


## Crates using `SelfRefCol`

The following crates use `SelfRefCol` to conveniently build the corresponding data structure:
* [orx-linked-list](https://crates.io/crates/orx-linked-list): implements singly and doubly linked lists.

## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be improved, please open an [issue](https://github.com/orxfun/orx-selfref-col/issues/new) or create a PR.

## License

This library is licensed under MIT license. See LICENSE for details.
