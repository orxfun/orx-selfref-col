use crate::Refs;

/// Variant defining `SelfRefCol` specifications.
pub trait Variant: Sized {
    /// Elements of the collection.
    type Item;

    /// The way the previous node references will be stored.
    /// * `RefsNone` if there is no reference.
    /// * `RefsSingle` if there is zero or one reference.
    /// * `RefsArray` if there is a constant number of references.
    /// * `RefsVec` if there is a dynamic number of references.
    type Prev: Refs;

    /// The way the next node references will be stored.
    /// * `RefsNone` if there is no reference.
    /// * `RefsSingle` if there is zero or one reference.
    /// * `RefsArray` if there is a constant number of references.
    /// * `RefsVec` if there is a dynamic number of references.
    type Next: Refs;

    /// The way the ends of the collection will be stored,
    /// such as the front of a linked list or root of a tree.
    /// * `RefsNone` if there is no reference.
    /// * `RefsSingle` if there is zero or one reference.
    /// * `RefsArray` if there is a constant number of references.
    /// * `RefsVec` if there is a dynamic number of references.
    type Ends: Refs;
}
