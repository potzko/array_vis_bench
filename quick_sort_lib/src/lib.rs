//! Quick sort family — single-pivot + dual-pivot, with deferred-small-sort
//! variants of each. Family declarations live in this crate's
//! `Cargo.toml`; the 30 standalone-partition (P × V) registrations live
//! in [`partitions_standalone`].

pub mod deferred_dual_pivot_quick_sort;
pub mod deferred_quick_sort;
pub mod dual_pivot_quick_sort;
pub mod partitions_standalone;
pub mod pivot_selectors;
pub mod quick_sort;

pub use deferred_dual_pivot_quick_sort::DeferredDualPivotQuickSort;
pub use deferred_quick_sort::DeferredQuickSort;
pub use dual_pivot_quick_sort::DualPivotQuickSort;
pub use pivot_selectors::{CombinedSelector, NintherDualPivot};
pub use quick_sort::QuickSort;
