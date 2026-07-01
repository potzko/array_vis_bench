//! Quick sort family. [`CanonicalQuickSort<P, SS>`] is the single pivot-free
//! driver: `P` is a self-splitting `PartitionScheme` (`N_PIVOTS = 0`) and the
//! sort carries no pivot of its own. Pivot-based quicksorts wrap a scheme +
//! selector as [`Pivoted<Scheme, Sel>`] — single-pivot (`LeftLeftPartition` + a
//! [`PivotSelector`]) or dual-pivot ([`yaroslavskiy::DualPivotPartition`] + a
//! `DualPivotSelector` like [`CombinedSelector`] / [`NintherDualPivot`]). The
//! catalog emits `CanonicalQuickSort<Pivoted<…>, SS>` directly, and the same
//! `Pivoted<…>` partitions are the workspace's first-class Partition kind — one
//! partition role-system, not two. Family declarations live in this crate's
//! `Cargo.toml`.
//!
//! [`PivotSelector`]: array_vis_bench_traits::PivotSelector

pub mod deferred_quick_sort;
pub mod pivot_selectors;
pub mod pivoted;
pub mod quick_sort;
pub mod yaroslavskiy;

pub use deferred_quick_sort::DeferredCanonicalQuickSort;
pub use pivot_selectors::{CombinedSelector, NintherDualPivot};
pub use pivoted::Pivoted;
pub use quick_sort::CanonicalQuickSort;
pub use yaroslavskiy::DualPivotPartition;
