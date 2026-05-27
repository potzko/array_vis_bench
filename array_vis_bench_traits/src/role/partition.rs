//! `PartitionScheme` — partition algorithm role.
//!
//! Implemented by leaf crates like `partition_lomuto` so they can stay
//! tiny and reference only this trait crate + `sort_logger`. The wiring
//! crate (`array_vis_bench_full`) consumes the trait via the partition
//! family's `[[package.metadata.array_vis_bench.families]]` cross-product.
//!
//! Each `partition` call announces its unsorted regions through a
//! [`PartitionVisitor`]; sorted regions (the pivots themselves, any
//! equal-pinned runs) are the implicit gaps between successive
//! `unsorted` calls. The trait generalises over pivot arity via
//! [`PartitionScheme::N_PIVOTS`] so single-pivot impls (LeftLeftPartition, LeftRightPartition,
//! ThreeWay, MovingPivot, Block, …) and dual-pivot impls
//! (DualPivotPartition) share one surface.

use core::ops::Range;
use sort_logger::SortLogger;

/// Receives the unsorted regions a partition leaves behind. The
/// quicksort driver implements this to schedule each region for
/// recursive sorting; sorted regions are the gaps between calls.
pub trait PartitionVisitor {
    /// A region of the array that still needs sorting.
    fn unsorted(&mut self, range: Range<usize>);
}

/// One partition step. Implementors take an array and a slice of
/// pre-selected pivot indices, do the actual swapping work, and
/// announce each unsorted region they leave behind through `visitor`.
///
/// Generalised over pivot arity via [`Self::N_PIVOTS`] so the same
/// trait covers single-pivot partitions (`N_PIVOTS = 1`) and
/// dual-pivot partitions (`N_PIVOTS = 2`) uniformly.
pub trait PartitionScheme {
    /// Display name used in the registry's menu path.
    const NAME: &'static str;
    /// Pivot arity. The caller supplies a `pivots` slice of exactly
    /// this length per call.
    const N_PIVOTS: usize;
    /// Partition `arr` using the pivots originally at the indices in
    /// `pivots` (length == [`Self::N_PIVOTS`]). Each unsorted region
    /// is announced to `visitor`; placed regions are the implicit
    /// gaps between calls and don't need to be reported.
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        visitor: &mut V,
    )
    where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor;
}
