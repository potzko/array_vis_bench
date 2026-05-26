//! Heap comparison — selects min-heap vs max-heap ordering.

use sort_logger::SortLogger;

pub trait Compare {
    /// Returns true if `arr[a]` should be more "rootward" than `arr[b]`.
    /// Min: `arr[a] < arr[b]`. Max: `arr[a] > arr[b]`. Indices are physical.
    fn comes_first<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        logger: &mut U,
        arr: &[T],
        a: usize,
        b: usize,
    ) -> bool;

    /// `arr[a]` is more-or-equally rootward than `arr[b]`. Min: `arr[a] ≤
    /// arr[b]`. Max: `arr[a] ≥ arr[b]`. Used by partition variants whose
    /// non-strict equivalent of `comes_first` matters for grouping equal
    /// keys (Lomuto, Block) the same way the standard Ord-based partitions
    /// do.
    fn comes_first_or_eq<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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

    #[inline(always)]
    fn comes_first_or_eq<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        logger: &mut U,
        arr: &[T],
        a: usize,
        b: usize,
    ) -> bool {
        logger.cmp_le(arr, a, b)
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

    #[inline(always)]
    fn comes_first_or_eq<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        logger: &mut U,
        arr: &[T],
        a: usize,
        b: usize,
    ) -> bool {
        logger.cmp_ge(arr, a, b)
    }
}
