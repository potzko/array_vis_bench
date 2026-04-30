//! Heap comparison — selects min-heap vs max-heap ordering.

use crate::traits::log_traits::SortLogger;

pub trait Compare {
    /// Returns true if `arr[a]` should be more "rootward" than `arr[b]`.
    /// Min: `arr[a] < arr[b]`. Max: `arr[a] > arr[b]`. Indices are physical.
    fn comes_first<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        logger: &mut U,
        arr: &[T],
        a: usize,
        b: usize,
    ) -> bool;
}

pub struct Min;
impl Compare for Min {
    #[inline(always)]
    fn comes_first<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        logger: &mut U,
        arr: &[T],
        a: usize,
        b: usize,
    ) -> bool {
        logger.cmp_lt(arr, a, b)
    }
}

pub struct Max;
impl Compare for Max {
    #[inline(always)]
    fn comes_first<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        logger: &mut U,
        arr: &[T],
        a: usize,
        b: usize,
    ) -> bool {
        logger.cmp_gt(arr, a, b)
    }
}
