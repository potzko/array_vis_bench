//! Lomuto partition (left-left single-pointer scan).
//!
//! Phase 3 pilot leaf crate. The struct + `PartitionScheme` impl + the
//! composable annotations (`HasTimeBounds` / `HasSpace` / `HasStability`)
//! all live here so the wiring crate (`array_vis_bench`) doesn't carry
//! Lomuto's source at all — just a dep edge.
//!
//! `array_vis_bench` re-exports `Lomuto` from
//! `crate::sorts::quick_sorts::partitions::Lomuto` so callsites and the
//! family! `uses` block don't move. Component discovery happens via the
//! `[[package.metadata.array_vis_bench.components]]` block in this
//! crate's `Cargo.toml`.

use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionSchemeV,
    PartitionVisitor,
};
use sort_logger::SortLogger;

/// Lomuto partition (left-left single-pointer scan).
///
/// Moves the pivot to the end, scans left-to-right placing small elements
/// at the front, then swaps the pivot into its final position.
pub struct Lomuto;

impl PartitionScheme for Lomuto {
    const NAME: &'static str = "lomuto";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        let len = arr.len();
        logger.swap(arr, pivot_idx, len - 1);
        let pivot = arr[len - 1];

        let mut small = 0;
        for i in 0..len - 1 {
            if logger.cmp_le_data(arr, i, pivot) {
                logger.swap(arr, i, small);
                small += 1;
            }
        }
        logger.swap(arr, small, len - 1);
        (small, small + 1)
    }
}

// ── Visitor-pattern impl (A/B prototype) ────────────────────────────────────
//
// Same algorithm body as `PartitionScheme::partition` — only the return
// shape differs. With `#[inline]` on the trait method, the two
// `visitor.unsorted(...)` calls should lower to the same `mov`+`call`
// pattern as the tuple-return version after monomorphisation.

impl PartitionSchemeV for Lomuto {
    const NAME: &'static str = "lomuto";
    const N_PIVOTS: usize = 1;
    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let len = arr.len();
        logger.swap(arr, pivots[0], len - 1);
        let pivot = arr[len - 1];

        let mut small = 0;
        for i in 0..len - 1 {
            if logger.cmp_le_data(arr, i, pivot) {
                logger.swap(arr, i, small);
                small += 1;
            }
        }
        logger.swap(arr, small, len - 1);
        visitor.unsorted(0..small);
        visitor.unsorted(small + 1..len);
    }
}

// Single-pass partition: O(N) time, O(1) aux space, not stable. Same
// values as the original `impl_partition_annotations!` macro in the host
// file — written out long-hand here so the leaf doesn't depend on a
// host-crate macro.
impl HasTimeBounds for Lomuto {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for Lomuto {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Lomuto {
    const STABLE: bool = false;
}
