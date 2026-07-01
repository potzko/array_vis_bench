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

/// The dual-pivot self-pivoting partition — exactly `Pivoted<DualPivotPartition,
/// Sel>` with the dual scheme baked in, leaving only the dual selector `Sel` open.
///
/// It exists purely to give the dual composite a DISTINCT type head from the
/// single `Pivoted<Scheme, Sel>`: the AVBS query language resolves component
/// references by type-head, and `pk_single` (`Pivoted<{scheme},{pivot}>`) and
/// `pk_dual` would otherwise be indistinguishable. As a plain alias it inherits
/// every `Pivoted` impl (`PartitionScheme` + the composable triad), so
/// `CanonicalQuickSort<DualPivoted<Sel>, SS>` and `RecursiveQuickSelect<DualPivoted<Sel>>`
/// compile unchanged.
pub type DualPivoted<Sel> = Pivoted<DualPivotPartition, Sel>;
