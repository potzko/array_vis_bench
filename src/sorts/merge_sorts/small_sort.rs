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
