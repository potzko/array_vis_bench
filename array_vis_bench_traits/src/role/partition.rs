//! `PartitionScheme` — partition algorithm role.
//!
//! Implemented by leaf crates like `partition_lomuto` so they can stay
//! tiny and reference only this trait crate + `sort_logger`. The wiring
//! crate (`array_vis_bench`) consumes the trait via its
//! `#[package.metadata.array_vis_bench.components]` cross-product.

use core::ops::Range;
use sort_logger::SortLogger;

pub trait PartitionScheme {
    /// Display name used both in the `Partition` component slot and in
    /// the per-algorithm path the menu builds at startup.
    const NAME: &'static str;
    /// Partition `arr` with the pivot originally at `pivot_idx`.
    ///
    /// Returns `(left_end, right_start)`:
    /// - `arr[..left_end]` needs further sorting
    /// - `arr[right_start..]` needs further sorting
    /// - `arr[left_end..right_start]` is already placed
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize);
}

// ────────────────────────────────────────────────────────────────────────────
//  Visitor-pattern prototype (A/B candidate for the current trait).
//  Each partition pushes its unsorted regions into a visitor instead of
//  returning a fixed-shape `(usize, usize)`. The shape generalises to
//  any number of regions (2-way → 2 calls, 3-way → 2 calls, dual-pivot
//  eq-pinning → 3 calls) without changing the trait method signature.
// ────────────────────────────────────────────────────────────────────────────

/// Receives the unsorted regions a partition leaves behind. The sort
/// driver implements this to schedule each region for recursive sorting.
/// Sorted regions (pivot, eq-runs) are *implicit* — they're the gaps
/// between successive `unsorted` calls.
pub trait PartitionVisitor {
    /// A region that still needs sorting.
    fn unsorted(&mut self, range: Range<usize>);
}

/// Visitor-pattern variant of [`PartitionScheme`]. Same role, different
/// return shape — kept side-by-side so the two can be A/B benchmarked
/// without affecting the live trait. Generalised over pivot arity via
/// [`Self::N_PIVOTS`] so the same trait covers single-pivot (Lomuto,
/// Hoare, …) and dual-pivot (Yaroslavskiy) partitions uniformly.
pub trait PartitionSchemeV {
    /// Display name (same role as [`PartitionScheme::NAME`]).
    const NAME: &'static str;
    /// Pivot arity: number of pivot indices the partition consumes per
    /// call. `1` for single-pivot, `2` for dual-pivot, etc. The caller
    /// is responsible for supplying a `pivots` slice of exactly this
    /// length.
    const N_PIVOTS: usize;
    /// Partition `arr` using the pivots originally at the indices in
    /// `pivots` (length == [`Self::N_PIVOTS`]). Each unsorted region is
    /// announced to `visitor`; placed regions (the pivots themselves,
    /// any equal-pinned runs) are the gaps between calls and don't
    /// need to be reported explicitly.
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
