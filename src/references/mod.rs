/// Iterators over node references.
pub mod iter;

mod array;
mod array_left_most;
mod node_idx;
mod node_idx_error;
mod node_ptr;
mod none;
mod refs;
mod single;
mod vec;

pub use array::RefsArray;
pub use array_left_most::RefsArrayLeftMost;
pub use node_idx::NodeIdx;
pub use node_idx_error::NodeIdxError;
pub use node_ptr::NodePtr;
pub use none::RefsNone;
pub use refs::Refs;
pub use single::RefsSingle;
pub use vec::RefsVec;
