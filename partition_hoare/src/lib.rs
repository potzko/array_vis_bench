use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionSchemeV,
    PartitionVisitor,
};
use sort_logger::SortLogger;

/// Hoare partition (left-right two-pointer scan).
///
/// Moves the pivot to the start, scans inward from both ends, then swaps
/// the pivot into its final position.
pub struct Hoare;

impl PartitionScheme for Hoare {
    const NAME: &'static str = "hoare";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);
        let pivot = arr[0];

        let mut left = 1;
        let mut right = arr.len() - 1;
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
        (right, right + 1)
    }
}

impl PartitionSchemeV for Hoare {
    const NAME: &'static str = "hoare";
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
impl HasTimeBounds for Hoare {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for Hoare {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Hoare {
    const STABLE: bool = false;
}
