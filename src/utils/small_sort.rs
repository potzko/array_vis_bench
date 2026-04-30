use crate::traits::log_traits::SortLogger;

/// Insertion sort. Returns `true` if the array was mutated (any swap
/// happened), `false` if it was already sorted.
#[inline(always)]
pub(crate) fn insertion_sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let mut mutated = false;
    for i in 1..arr.len() {
        let mut ii = i;
        while ii > 0 && logger.cond_swap_lt(arr, ii, ii - 1) {
            mutated = true;
            ii -= 1;
        }
    }
    mutated
}

/// Strategy for sorting small sub-arrays before or during a merge / quick /
/// shell pass.
///
/// # Contract
///
/// Implementors expose a compile-time `THRESHOLD`. If 0 the small sort is
/// never triggered; otherwise [`Self::sort`] **must produce a fully sorted
/// output for any `arr.len() <= THRESHOLD`** — including 0, 1, 2, … up to
/// and including `THRESHOLD`. Callers may pass any length within that range
/// (some pad/clamp at boundaries), so a network-style impl that only sorts
/// at exactly its peak size must fall back to insertion sort (or equivalent)
/// for smaller inputs.
///
/// `sort` returns `true` if the array was mutated (any swap happened),
/// `false` if it was already sorted. All impls are `#[inline(always)]` so a
/// caller that discards the bool gets the change-tracking dead-code-eliminated
/// by the compiler.
pub trait SmallSort {
    /// Subarray length at or below which this strategy is invoked (0 = never).
    const THRESHOLD: usize;

    /// Sort `arr` in-place. Caller guarantees `arr.len() <= Self::THRESHOLD`;
    /// implementor guarantees correct sorted output for **any** such length.
    ///
    /// Returns `true` if the array was mutated, `false` if it was already sorted.
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool;
}

/// Subtrait of [`SmallSort`] for variants whose threshold is strictly
/// above 1 — i.e. they actually do sorting work for arrays larger than a
/// single element. Use this bound on sorts whose algorithm relies on the
/// small sort to make meaningful progress (e.g. block-level odd-even).
pub trait NonTrivialSmallSort: SmallSort {}

// ---------------------------------------------------------------------------

/// No small-sort: recurse / iterate all the way down to subarrays of size 1.
pub struct NoSmallSort;
combo_codegen::component!(SmallSort, NoSmallSort, "no threshold");

impl SmallSort for NoSmallSort {
    const THRESHOLD: usize = 0;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) -> bool {
        unreachable!("NoSmallSort::sort should never be called (THRESHOLD = 0)")
    }
}

// ---------------------------------------------------------------------------

/// Trivial small-sort: arrays of length ≤ 1 are already sorted; do nothing.
pub struct Size1SmallSort;
combo_codegen::component!(SmallSort, Size1SmallSort, "size: 1");

impl SmallSort for Size1SmallSort {
    const THRESHOLD: usize = 1;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------

/// Small-sort for arrays of length ≤ 2: single conditional swap when len == 2.
pub struct Size2SmallSort;
combo_codegen::component!(SmallSort, Size2SmallSort, "size: 2");
combo_codegen::component!(NonTrivialSmallSort, Size2SmallSort, "size: 2");

impl SmallSort for Size2SmallSort {
    const THRESHOLD: usize = 2;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        if arr.len() == 2 {
            logger.cond_swap_gt(arr, 0, 1)
        } else {
            false
        }
    }
}
impl NonTrivialSmallSort for Size2SmallSort {}

// ---------------------------------------------------------------------------

/// Insertion sort for subarrays of length ≤ N.
pub struct InsertionSmallSort<const N: usize>;
combo_codegen::component!(SmallSort, InsertionSmallSort<16>, "insertion: 16");
combo_codegen::component!(SmallSort, InsertionSmallSort<32>, "insertion: 32");
combo_codegen::component!(NonTrivialSmallSort, InsertionSmallSort<16>, "insertion: 16");
combo_codegen::component!(NonTrivialSmallSort, InsertionSmallSort<32>, "insertion: 32");

impl<const N: usize> SmallSort for InsertionSmallSort<N> {
    const THRESHOLD: usize = N;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        insertion_sort(arr, logger)
    }
}
impl<const N: usize> NonTrivialSmallSort for InsertionSmallSort<N> {}

// ---------------------------------------------------------------------------

/// Optimal sorting network for subarrays of length ≤ 8.
///
/// Uses 19 compare-and-swap operations (optimal) when len == 8.
/// Falls back to insertion sort for smaller sizes.
pub struct NetworkSmallSort;
combo_codegen::component!(SmallSort, NetworkSmallSort, "network: 8");
combo_codegen::component!(NonTrivialSmallSort, NetworkSmallSort, "network: 8");

impl SmallSort for NetworkSmallSort {
    const THRESHOLD: usize = 8;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        if arr.len() == 8 {
            sort_network_8(arr, logger)
        } else {
            insertion_sort(arr, logger)
        }
    }
}
impl NonTrivialSmallSort for NetworkSmallSort {}

// ---------------------------------------------------------------------------

/// Sorting network for subarrays of length ≤ 16.
///
/// Uses the optimal 19-comparator network for size 8 and Batcher's odd-even
/// merge sort network (63 comparators) for size 16.
/// Falls back to insertion sort for other sizes.
pub struct Network16SmallSort;
combo_codegen::component!(SmallSort, Network16SmallSort, "network: 16");
combo_codegen::component!(NonTrivialSmallSort, Network16SmallSort, "network: 16");

impl SmallSort for Network16SmallSort {
    const THRESHOLD: usize = 16;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        match arr.len() {
            16 => sort_network_16(arr, logger),
            8 => sort_network_8(arr, logger),
            _ => insertion_sort(arr, logger),
        }
    }
}
impl NonTrivialSmallSort for Network16SmallSort {}

// ---------------------------------------------------------------------------

/// Optimal 8-element sorting network (19 comparators, 6 stages).
/// Returns `true` if the array was mutated.
#[inline(always)]
fn sort_network_8<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
    let mut mutated = false;
    // Stage 1
    mutated |= logger.cond_swap_gt(arr, 0, 1);
    mutated |= logger.cond_swap_gt(arr, 2, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 5);
    mutated |= logger.cond_swap_gt(arr, 6, 7);
    // Stage 2
    mutated |= logger.cond_swap_gt(arr, 0, 2);
    mutated |= logger.cond_swap_gt(arr, 1, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 6);
    mutated |= logger.cond_swap_gt(arr, 5, 7);
    // Stage 3
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    // Stage 4
    mutated |= logger.cond_swap_gt(arr, 0, 4);
    mutated |= logger.cond_swap_gt(arr, 1, 5);
    mutated |= logger.cond_swap_gt(arr, 2, 6);
    mutated |= logger.cond_swap_gt(arr, 3, 7);
    // Stage 5
    mutated |= logger.cond_swap_gt(arr, 2, 4);
    mutated |= logger.cond_swap_gt(arr, 3, 5);
    // Stage 6
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 3, 4);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated
}

/// Batcher's odd-even merge sort network for 16 elements (63 comparators, 10 stages).
/// Returns `true` if the array was mutated.
#[inline(always)]
fn sort_network_16<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
    let mut mutated = false;
    // Stage 1: sort pairs
    mutated |= logger.cond_swap_gt(arr, 0, 1);
    mutated |= logger.cond_swap_gt(arr, 2, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 5);
    mutated |= logger.cond_swap_gt(arr, 6, 7);
    mutated |= logger.cond_swap_gt(arr, 8, 9);
    mutated |= logger.cond_swap_gt(arr, 10, 11);
    mutated |= logger.cond_swap_gt(arr, 12, 13);
    mutated |= logger.cond_swap_gt(arr, 14, 15);
    // Stage 2: merge pairs → sorted 4s (even step)
    mutated |= logger.cond_swap_gt(arr, 0, 2);
    mutated |= logger.cond_swap_gt(arr, 1, 3);
    mutated |= logger.cond_swap_gt(arr, 4, 6);
    mutated |= logger.cond_swap_gt(arr, 5, 7);
    mutated |= logger.cond_swap_gt(arr, 8, 10);
    mutated |= logger.cond_swap_gt(arr, 9, 11);
    mutated |= logger.cond_swap_gt(arr, 12, 14);
    mutated |= logger.cond_swap_gt(arr, 13, 15);
    // Stage 3: merge pairs → sorted 4s (fixup)
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated |= logger.cond_swap_gt(arr, 9, 10);
    mutated |= logger.cond_swap_gt(arr, 13, 14);
    // Stage 4: merge sorted 4s → sorted 8s (even step)
    mutated |= logger.cond_swap_gt(arr, 0, 4);
    mutated |= logger.cond_swap_gt(arr, 1, 5);
    mutated |= logger.cond_swap_gt(arr, 2, 6);
    mutated |= logger.cond_swap_gt(arr, 3, 7);
    mutated |= logger.cond_swap_gt(arr, 8, 12);
    mutated |= logger.cond_swap_gt(arr, 9, 13);
    mutated |= logger.cond_swap_gt(arr, 10, 14);
    mutated |= logger.cond_swap_gt(arr, 11, 15);
    // Stage 5: merge sorted 4s → sorted 8s (odd step)
    mutated |= logger.cond_swap_gt(arr, 2, 4);
    mutated |= logger.cond_swap_gt(arr, 3, 5);
    mutated |= logger.cond_swap_gt(arr, 10, 12);
    mutated |= logger.cond_swap_gt(arr, 11, 13);
    // Stage 6: merge sorted 4s → sorted 8s (fixup)
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 3, 4);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated |= logger.cond_swap_gt(arr, 9, 10);
    mutated |= logger.cond_swap_gt(arr, 11, 12);
    mutated |= logger.cond_swap_gt(arr, 13, 14);
    // Stage 7: merge sorted 8s → sorted 16 (even step)
    mutated |= logger.cond_swap_gt(arr, 0, 8);
    mutated |= logger.cond_swap_gt(arr, 1, 9);
    mutated |= logger.cond_swap_gt(arr, 2, 10);
    mutated |= logger.cond_swap_gt(arr, 3, 11);
    mutated |= logger.cond_swap_gt(arr, 4, 12);
    mutated |= logger.cond_swap_gt(arr, 5, 13);
    mutated |= logger.cond_swap_gt(arr, 6, 14);
    mutated |= logger.cond_swap_gt(arr, 7, 15);
    // Stage 8: merge sorted 8s → sorted 16 (odd step)
    mutated |= logger.cond_swap_gt(arr, 4, 8);
    mutated |= logger.cond_swap_gt(arr, 5, 9);
    mutated |= logger.cond_swap_gt(arr, 6, 10);
    mutated |= logger.cond_swap_gt(arr, 7, 11);
    // Stage 9: merge sorted 8s → sorted 16 (fixup 1)
    mutated |= logger.cond_swap_gt(arr, 2, 4);
    mutated |= logger.cond_swap_gt(arr, 3, 5);
    mutated |= logger.cond_swap_gt(arr, 6, 8);
    mutated |= logger.cond_swap_gt(arr, 7, 9);
    mutated |= logger.cond_swap_gt(arr, 10, 12);
    mutated |= logger.cond_swap_gt(arr, 11, 13);
    // Stage 10: merge sorted 8s → sorted 16 (fixup 2)
    mutated |= logger.cond_swap_gt(arr, 1, 2);
    mutated |= logger.cond_swap_gt(arr, 3, 4);
    mutated |= logger.cond_swap_gt(arr, 5, 6);
    mutated |= logger.cond_swap_gt(arr, 7, 8);
    mutated |= logger.cond_swap_gt(arr, 9, 10);
    mutated |= logger.cond_swap_gt(arr, 11, 12);
    mutated |= logger.cond_swap_gt(arr, 13, 14);
    mutated
}

// ---------------------------------------------------------------------------

/// Strategy for sorting a sub-array of a *fixed* compile-time size `N`.
///
/// Unlike [`SmallSort`] — which guarantees correctness for any length up to
/// its threshold — implementors of `SetSizeSmallSort<N>` only handle arrays
/// of exactly `N` elements. The size is enforced at the type level via
/// `&mut [T; N]`; callers convert from a slice with `<&mut [T; N]>::try_from`
/// (or destructure with array patterns) after aligning the input.
pub trait SetSizeSmallSort<const N: usize> {
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T; N], logger: &mut U);
}

/// Adapter exposing any [`SmallSort`] as a [`SetSizeSmallSort<N>`] for every
/// `N <= S::THRESHOLD`. The bound is enforced at compile time via an inline
/// `const` assertion. The mutation bool returned by the underlying sort is
/// discarded — fixed-size callers don't need it.
pub struct SmallSortAdapter<S: SmallSort>(std::marker::PhantomData<S>);

impl<const N: usize, S: SmallSort> SetSizeSmallSort<N> for SmallSortAdapter<S> {
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T; N], logger: &mut U) {
        const {
            assert!(
                N <= S::THRESHOLD,
                "SmallSortAdapter<S>: N must be <= S::THRESHOLD",
            );
        }
        let _ = S::sort(arr, logger);
    }
}

// ---------------------------------------------------------------------------

/// Marks a threshold at which quicksort stops recursing, leaving small
/// sub-arrays unsorted. After the full recursion the caller runs a single
/// insertion sort pass over the entire array.
pub trait DeferredSmallSort {
    const THRESHOLD: usize;
}

/// Deferred-insertion threshold of N: quicksort stops at sub-arrays of
/// length ≤ N and lets a final insertion sort pass clean up.
pub struct DeferredInsertion<const N: usize>;
combo_codegen::component!(DeferredSmallSort, DeferredInsertion<16>, "deferred insertion: 16");
combo_codegen::component!(DeferredSmallSort, DeferredInsertion<32>, "deferred insertion: 32");

impl<const N: usize> DeferredSmallSort for DeferredInsertion<N> {
    const THRESHOLD: usize = N;
}
