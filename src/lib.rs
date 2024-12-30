#![doc = include_str!("../README.md")]
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
#![no_std]
extern crate alloc;

/// Node references.
pub mod references;

mod common_traits;
mod core_col;
mod memory;
mod node;
mod selfref_col;
mod variant;

pub use core_col::CoreCol;
pub use memory::{
    MemoryPolicy, MemoryReclaimNever, MemoryReclaimOnThreshold, MemoryReclaimer, MemoryState,
    Utilization,
};
pub use node::Node;
pub use references::{NodeIdx, NodeIdxError, NodePtr};
pub use references::{Refs, RefsArray, RefsArrayLeftMost, RefsNone, RefsSingle, RefsVec};
pub use selfref_col::SelfRefCol;
pub use variant::Variant;
