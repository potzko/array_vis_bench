//! Quick sort family. The unified [`QuickSort<P, V, SS>`] handles both
//! single-pivot (`P::N_PIVOTS = 1`, e.g. `Lomuto` + a [`PivotSelector`])
//! and dual-pivot (`P::N_PIVOTS = 2`, e.g. [`yaroslavskiy::Yaroslavskiy`]
//! + a `DualPivotSelector` like [`CombinedSelector`] or
//! [`NintherDualPivot`]) variants — the old standalone
//! `DualPivotQuickSort` is gone, replaced by `QuickSort<Yaroslavskiy,
//! <DPS>, <SS>>`. Family declarations live in this crate's `Cargo.toml`.
//! The 30 standalone `(P × V)` partition registrations live in the
//! sibling `quick_partition_registry` crate so this leaf doesn't drag
//! every partition + pivot leaf into every consumer.
//!
//! [`PivotSelector`]: array_vis_bench_traits::PivotSelector

pub mod deferred_quick_sort;
pub mod pivot_selectors;
pub mod quick_sort;
pub mod yaroslavskiy;

pub use deferred_quick_sort::DeferredQuickSort;
pub use pivot_selectors::{CombinedSelector, NintherDualPivot};
pub use quick_sort::QuickSort;
pub use yaroslavskiy::Yaroslavskiy;
