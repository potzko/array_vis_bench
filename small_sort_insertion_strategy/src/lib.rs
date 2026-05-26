//! Two strategies for placing one element of a sorted prefix.

use array_vis_bench_traits::InsertionStrategy;
use sort_logger::SortLogger;

/// Linear insertion: walk left, swapping each out-of-order pair until
/// the element settles. `O(d)` work where `d` is the displacement.
pub struct LinearInsertion;

impl InsertionStrategy for LinearInsertion {
    #[inline(always)]
    fn insert_one<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        logger: &mut U,
    ) -> bool {
        let mut mutated = false;
        let mut ii = i;
        while ii > 0 && logger.cond_swap_lt(arr, ii, ii - 1) {
            mutated = true;
            ii -= 1;
        }
        mutated
    }
}

/// Binary insertion: binary-search the sorted prefix for the destination,
/// then shift the gap open with adjacent swaps. `O(log d)` compares,
/// still `O(d)` swaps.
pub struct BinaryInsertion;

impl InsertionStrategy for BinaryInsertion {
    #[inline(always)]
    fn insert_one<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        logger: &mut U,
    ) -> bool {
        let mut lo = 0;
        let mut hi = i;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if logger.cmp_gt(arr, mid, i) {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        if lo == i {
            return false;
        }
        let mut ii = i;
        while ii > lo {
            logger.swap(arr, ii, ii - 1);
            ii -= 1;
        }
        true
    }
}
