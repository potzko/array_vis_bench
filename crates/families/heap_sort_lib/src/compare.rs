//! Heap comparison — selects min-heap vs max-heap ordering.

use sort_logger::SortLogger;

pub trait Compare {
    /// `true` for `Min` (rootward = smaller `Ord`), `false` for `Max`
    /// (rootward = larger `Ord`). Lets a `PartitionScheme`-driven heap
    /// build pick the natural sort direction at compile time without an
    /// extra value comparison.
    const ROOTWARD_IS_SMALLER_ORD: bool;

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
    /// keys (LeftLeftPartition, Block) the same way the standard Ord-based partitions
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
    const ROOTWARD_IS_SMALLER_ORD: bool = true;

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
    const ROOTWARD_IS_SMALLER_ORD: bool = false;

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
