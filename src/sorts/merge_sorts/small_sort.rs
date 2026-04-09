use crate::traits::log_traits::SortLogger;
use super::utils::insertion_sort;

/// Strategy for sorting small sub-arrays before or during a merge sort pass.
///
/// Implementors expose a compile-time `THRESHOLD`: if 0 the small sort is
/// never triggered; otherwise it applies to subarrays of length ≤ THRESHOLD.
pub trait SmallSort {
    /// Subarray length at or below which this strategy is invoked (0 = never).
    const THRESHOLD: usize;

    /// Sort `arr` in-place. Only called when `arr.len() <= Self::THRESHOLD`.
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U);
}

// ---------------------------------------------------------------------------

/// No small-sort: recurse / iterate all the way down to subarrays of size 1.
pub struct NoSmallSort;

impl SmallSort for NoSmallSort {
    const THRESHOLD: usize = 0;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) {
        unreachable!("NoSmallSort::sort should never be called (THRESHOLD = 0)")
    }
}

// ---------------------------------------------------------------------------

/// Insertion sort for subarrays of length ≤ N.
pub struct InsertionSmallSort<const N: usize>;

impl<const N: usize> SmallSort for InsertionSmallSort<N> {
    const THRESHOLD: usize = N;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        insertion_sort(arr, logger);
    }
}

// ---------------------------------------------------------------------------

/// Optimal sorting network for subarrays of length ≤ 8.
///
/// Uses 19 compare-and-swap operations (optimal) when len == 8.
/// Falls back to insertion sort for smaller sizes.
pub struct NetworkSmallSort;

impl SmallSort for NetworkSmallSort {
    const THRESHOLD: usize = 8;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() == 8 {
            sort_network_8(arr, logger);
        } else {
            insertion_sort(arr, logger);
        }
    }
}

/// Sorting network for subarrays of length ≤ 16.
///
/// Uses the optimal 19-comparator network for size 8 and Batcher's odd-even
/// merge sort network (63 comparators) for size 16.
/// Falls back to insertion sort for other sizes.
pub struct Network16SmallSort;

impl SmallSort for Network16SmallSort {
    const THRESHOLD: usize = 16;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        match arr.len() {
            16 => sort_network_16(arr, logger),
            8 => sort_network_8(arr, logger),
            _ => insertion_sort(arr, logger),
        }
    }
}

/// Optimal 8-element sorting network (19 comparators, 6 stages).
#[inline(always)]
fn sort_network_8<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    // Stage 1
    logger.cond_swap_gt(arr, 0, 1);
    logger.cond_swap_gt(arr, 2, 3);
    logger.cond_swap_gt(arr, 4, 5);
    logger.cond_swap_gt(arr, 6, 7);
    // Stage 2
    logger.cond_swap_gt(arr, 0, 2);
    logger.cond_swap_gt(arr, 1, 3);
    logger.cond_swap_gt(arr, 4, 6);
    logger.cond_swap_gt(arr, 5, 7);
    // Stage 3
    logger.cond_swap_gt(arr, 1, 2);
    logger.cond_swap_gt(arr, 5, 6);
    // Stage 4
    logger.cond_swap_gt(arr, 0, 4);
    logger.cond_swap_gt(arr, 1, 5);
    logger.cond_swap_gt(arr, 2, 6);
    logger.cond_swap_gt(arr, 3, 7);
    // Stage 5
    logger.cond_swap_gt(arr, 2, 4);
    logger.cond_swap_gt(arr, 3, 5);
    // Stage 6
    logger.cond_swap_gt(arr, 1, 2);
    logger.cond_swap_gt(arr, 3, 4);
    logger.cond_swap_gt(arr, 5, 6);
}

/// Batcher's odd-even merge sort network for 16 elements (63 comparators, 10 stages).
#[inline(always)]
fn sort_network_16<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    // Stage 1: sort pairs
    logger.cond_swap_gt(arr, 0, 1);
    logger.cond_swap_gt(arr, 2, 3);
    logger.cond_swap_gt(arr, 4, 5);
    logger.cond_swap_gt(arr, 6, 7);
    logger.cond_swap_gt(arr, 8, 9);
    logger.cond_swap_gt(arr, 10, 11);
    logger.cond_swap_gt(arr, 12, 13);
    logger.cond_swap_gt(arr, 14, 15);
    // Stage 2: merge pairs → sorted 4s (even step)
    logger.cond_swap_gt(arr, 0, 2);
    logger.cond_swap_gt(arr, 1, 3);
    logger.cond_swap_gt(arr, 4, 6);
    logger.cond_swap_gt(arr, 5, 7);
    logger.cond_swap_gt(arr, 8, 10);
    logger.cond_swap_gt(arr, 9, 11);
    logger.cond_swap_gt(arr, 12, 14);
    logger.cond_swap_gt(arr, 13, 15);
    // Stage 3: merge pairs → sorted 4s (fixup)
    logger.cond_swap_gt(arr, 1, 2);
    logger.cond_swap_gt(arr, 5, 6);
    logger.cond_swap_gt(arr, 9, 10);
    logger.cond_swap_gt(arr, 13, 14);
    // Stage 4: merge sorted 4s → sorted 8s (even step)
    logger.cond_swap_gt(arr, 0, 4);
    logger.cond_swap_gt(arr, 1, 5);
    logger.cond_swap_gt(arr, 2, 6);
    logger.cond_swap_gt(arr, 3, 7);
    logger.cond_swap_gt(arr, 8, 12);
    logger.cond_swap_gt(arr, 9, 13);
    logger.cond_swap_gt(arr, 10, 14);
    logger.cond_swap_gt(arr, 11, 15);
    // Stage 5: merge sorted 4s → sorted 8s (odd step)
    logger.cond_swap_gt(arr, 2, 4);
    logger.cond_swap_gt(arr, 3, 5);
    logger.cond_swap_gt(arr, 10, 12);
    logger.cond_swap_gt(arr, 11, 13);
    // Stage 6: merge sorted 4s → sorted 8s (fixup)
    logger.cond_swap_gt(arr, 1, 2);
    logger.cond_swap_gt(arr, 3, 4);
    logger.cond_swap_gt(arr, 5, 6);
    logger.cond_swap_gt(arr, 9, 10);
    logger.cond_swap_gt(arr, 11, 12);
    logger.cond_swap_gt(arr, 13, 14);
    // Stage 7: merge sorted 8s → sorted 16 (even step)
    logger.cond_swap_gt(arr, 0, 8);
    logger.cond_swap_gt(arr, 1, 9);
    logger.cond_swap_gt(arr, 2, 10);
    logger.cond_swap_gt(arr, 3, 11);
    logger.cond_swap_gt(arr, 4, 12);
    logger.cond_swap_gt(arr, 5, 13);
    logger.cond_swap_gt(arr, 6, 14);
    logger.cond_swap_gt(arr, 7, 15);
    // Stage 8: merge sorted 8s → sorted 16 (odd step)
    logger.cond_swap_gt(arr, 4, 8);
    logger.cond_swap_gt(arr, 5, 9);
    logger.cond_swap_gt(arr, 6, 10);
    logger.cond_swap_gt(arr, 7, 11);
    // Stage 9: merge sorted 8s → sorted 16 (fixup 1)
    logger.cond_swap_gt(arr, 2, 4);
    logger.cond_swap_gt(arr, 3, 5);
    logger.cond_swap_gt(arr, 6, 8);
    logger.cond_swap_gt(arr, 7, 9);
    logger.cond_swap_gt(arr, 10, 12);
    logger.cond_swap_gt(arr, 11, 13);
    // Stage 10: merge sorted 8s → sorted 16 (fixup 2)
    logger.cond_swap_gt(arr, 1, 2);
    logger.cond_swap_gt(arr, 3, 4);
    logger.cond_swap_gt(arr, 5, 6);
    logger.cond_swap_gt(arr, 7, 8);
    logger.cond_swap_gt(arr, 9, 10);
    logger.cond_swap_gt(arr, 11, 12);
    logger.cond_swap_gt(arr, 13, 14);
}
