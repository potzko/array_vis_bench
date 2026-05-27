//! Left-left single-pointer partition (historically the *Lomuto* scheme,
//! after Nico Lomuto).
//!
//! Phase 3 pilot leaf crate. The struct + `PartitionScheme` impl + the
//! composable annotations (`HasTimeBounds` / `HasSpace` / `HasStability`)
//! all live here so the wiring crate doesn't carry this partition's source
//! at all — just a dep edge. Component discovery happens via the
//! `[[package.metadata.array_vis_bench.components]]` block in this
//! crate's `Cargo.toml`.

use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionVisitor,
};
use sort_logger::SortLogger;

/// Left-left single-pointer partition — the *Lomuto* scheme (Nico Lomuto).
///
/// Moves the pivot to the end, scans left-to-right placing small elements
/// at the front, then swaps the pivot into its final position.
pub struct LeftLeftPartition;

impl PartitionScheme for LeftLeftPartition {
    const NAME: &'static str = "left-left pointer";
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

// Single-pass partition: O(N) time, O(1) aux space, not stable.
impl HasTimeBounds for LeftLeftPartition {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for LeftLeftPartition {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for LeftLeftPartition {
    const STABLE: bool = false;
}
