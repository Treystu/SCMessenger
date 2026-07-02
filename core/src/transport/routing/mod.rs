//! Re-exports from the crate-level routing module for backward compatibility.
//!
//! The routing module lives at `crate::routing` but `transport/mod.rs`
//! declares `pub mod routing`, so this shim re-exports everything.

pub use crate::routing::*;
