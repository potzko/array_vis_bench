use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionVisitor,
};
use sort_logger::SortLogger;

/// Left-right two-pointer (crossing) partition — the *Hoare* scheme
/// (C. A. R. Hoare).
///
/// Moves the pivot to the start, scans inward from both ends, then swaps
/// the pivot into its final position.
pub struct LeftRightPartition;

impl PartitionScheme for LeftRightPartition {
    const NAME: &'static str = "left-right pointer";
    const N_PIVOTS: usize = 1;
    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        _scratch: &mut [usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let len = arr.len();
        logger.swap(arr, pivots[0], 0);
        let pivot = arr[0];

        let mut left = 1;
        let mut right = len - 1;
        while left <= right {
            while left <= right && logger.cmp_le_data(arr, left, pivot) {
                left += 1;
            }
            while left <= right && logger.cmp_gt_data(arr, right, pivot) {
                right -= 1;
            }
            if left < right {
                logger.swap(arr, left, right);
                left += 1;
                right -= 1;
            }
        }
        logger.swap(arr, 0, right);
        visitor.unsorted(0..right);
        visitor.unsorted(right + 1..len);
    }
}

// Single-pass partition: O(N) time, O(1) aux space, not stable.
impl HasTimeBounds for LeftRightPartition {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for LeftRightPartition {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for LeftRightPartition {
    const STABLE: bool = false;
}
