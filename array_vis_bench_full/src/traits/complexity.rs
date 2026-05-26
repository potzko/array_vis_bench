//! Thin re-export shim. The actual `Complexity` / `Special` types and
//! their `const fn` API live in the `array_vis_bench_traits` workspace
//! crate so leaf-component crates can depend on them without pulling in
//! the full `array_vis_bench` tree.
//!
//! Kept here so existing `use crate::traits::complexity::Complexity`
//! paths continue to resolve unchanged.

pub use array_vis_bench_traits::complexity::*;
