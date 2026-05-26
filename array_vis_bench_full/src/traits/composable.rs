//! Thin re-export shim. The composable annotation traits
//! (`HasTimeBounds`, `HasSpace`, `HasStability`, `PivotQuality`) live in
//! the `array_vis_bench_traits` workspace crate so leaf-component crates
//! can implement them without depending on the full `array_vis_bench`
//! tree.
//!
//! Kept here so existing `use crate::traits::composable::*` paths
//! continue to resolve unchanged.

pub use array_vis_bench_traits::composable::*;
