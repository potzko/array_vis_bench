use std::marker::PhantomData;

use crate::traits::complexity::Complexity;
use crate::traits::composable::{HasSpace, HasStability, HasTimeBounds};
use crate::traits::log_traits::SortLogger;

// ---------------------------------------------------------------------------

/// Strategy for placing one element of a sorted prefix.
///
/// Given `arr[..i]` already sorted and `arr[i]` the new element, an
/// implementor moves `arr[i]` to its correct position within `arr[..=i]`.
/// Returns `true` if the array was mutated.
pub trait InsertionStrategy {
    fn insert_one<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        logger: &mut U,
    ) -> bool;
}

/// Linear insertion: walk left, swapping each out-of-order pair until the
/// element settles. `O(d)` work where `d` is the displacement.
pub struct LinearInsertion;
combo_codegen::component!(InsertionStrategy, LinearInsertion, "linear");

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
combo_codegen::component!(InsertionStrategy, BinaryInsertion, "binary");

impl InsertionStrategy for BinaryInsertion {
    #[inline(always)]
    fn insert_one<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        logger: &mut U,
    ) -> bool {
        // Find leftmost index `lo` in `0..i` with arr[lo] > arr[i].
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

/// Run a full insertion sort using the chosen [`InsertionStrategy`].
/// Returns `true` if any swap happened.
#[inline(always)]
pub fn insertion_sort_with<S: InsertionStrategy, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let mut mutated = false;
    for i in 1..arr.len() {
        mutated |= S::insert_one(arr, i, logger);
    }
    mutated
}

/// Linear insertion sort over the whole array. Kept as a free function
/// because several call sites (circle sorts, etc.) want it without
/// committing to a strategy parameter.
#[inline(always)]
pub(crate) fn insertion_sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    insertion_sort_with::<LinearInsertion, _, _>(arr, logger)
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

/// Register a `SmallSort` impl as a standalone algorithm. Sentinel
/// small-sorts (those with `THRESHOLD = 0`) should NOT call this —
/// they're glue, not algorithms.
macro_rules! register_small_sort {
    ($mod:ident, $ty:ty, $variant_name:expr) => {
        mod $mod {
            use super::*;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const NAME: &str = const_format::concatcp!("small-sort: ", $variant_name);

            fn sort_dyn(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                let _ = <$ty as crate::utils::small_sort::SmallSort>::sort(arr, logger);
            }
            fn sort_noop(arr: &mut [usize], logger: &mut NoOpLogger) {
                let _ = <$ty as crate::utils::small_sort::SmallSort>::sort(arr, logger);
            }

            fn run_with_input(
                input_name: &str,
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                // Clamp input size to the small-sort's declared
                // threshold; behaviour above-threshold is undefined per
                // the trait's contract.
                let threshold = <$ty as crate::utils::small_sort::SmallSort>::THRESHOLD;
                let clamped = crate::bench_registry::RunConfig {
                    size: config.size.min(threshold),
                    seed: config.seed,
                };
                crate::bench_registry::run_small_sort_with_input(
                    input_name, &clamped, sort_dyn, logger,
                );
            }

            fn run_correctness() {
                crate::bench_registry::correctness::small_sort_battery(
                    sort_noop,
                    NAME,
                    <$ty as crate::utils::small_sort::SmallSort>::THRESHOLD,
                );
            }

            // Small-sorts are bounded by THRESHOLD (compile-time const), so
            // their per-invocation time and space are O(1) regardless of
            // the algorithm's intrinsic complexity (e.g. insertion's
            // `HasTimeBounds::WORST = N²` is irrelevant inside a bounded
            // leaf — the outer composition substitutes CONST here). The
            // intrinsic `HasStability` impl still drives `stable`, since
            // stability is a structural property unaffected by bounding.
            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: crate::bench_registry::Category::SmallSort,
                    worst: crate::traits::complexity::Complexity::CONST,
                    best: crate::traits::complexity::Complexity::CONST,
                    average: crate::traits::complexity::Complexity::CONST,
                    space: crate::traits::complexity::Complexity::CONST,
                    stable: <$ty as crate::traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: Some(
                        <$ty as crate::utils::small_sort::SmallSort>::THRESHOLD,
                    ),
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    "O(K)",
                    false,
                    &["small-sorts", $variant_name],
                );
            }

            #[cfg(test)]
            mod small_sort_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                        &super::ENTRY,
                        crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                    );
                }
            }
        }
    };
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
register_small_sort!(register_size2, Size2SmallSort, "size: 2");

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

/// Insertion sort for subarrays of length ≤ N, dispatched via an
/// [`InsertionStrategy`] (linear or binary).
pub struct InsertionSmallSort<S: InsertionStrategy, const N: usize>(PhantomData<S>);
combo_codegen::component!(SmallSort, InsertionSmallSort<LinearInsertion, 16>, "insertion: 16");
combo_codegen::component!(SmallSort, InsertionSmallSort<LinearInsertion, 32>, "insertion: 32");
combo_codegen::component!(SmallSort, InsertionSmallSort<BinaryInsertion, 16>, "binary insertion: 16");
combo_codegen::component!(SmallSort, InsertionSmallSort<BinaryInsertion, 32>, "binary insertion: 32");
combo_codegen::component!(NonTrivialSmallSort, InsertionSmallSort<LinearInsertion, 16>, "insertion: 16");
combo_codegen::component!(NonTrivialSmallSort, InsertionSmallSort<LinearInsertion, 32>, "insertion: 32");
combo_codegen::component!(NonTrivialSmallSort, InsertionSmallSort<BinaryInsertion, 16>, "binary insertion: 16");
combo_codegen::component!(NonTrivialSmallSort, InsertionSmallSort<BinaryInsertion, 32>, "binary insertion: 32");
register_small_sort!(register_ins_linear_16, InsertionSmallSort<LinearInsertion, 16>, "insertion: 16");
register_small_sort!(register_ins_linear_32, InsertionSmallSort<LinearInsertion, 32>, "insertion: 32");
register_small_sort!(register_ins_binary_16, InsertionSmallSort<BinaryInsertion, 16>, "binary insertion: 16");
register_small_sort!(register_ins_binary_32, InsertionSmallSort<BinaryInsertion, 32>, "binary insertion: 32");

impl<S: InsertionStrategy, const N: usize> SmallSort for InsertionSmallSort<S, N> {
    const THRESHOLD: usize = N;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        insertion_sort_with::<S, _, _>(arr, logger)
    }
}
impl<S: InsertionStrategy, const N: usize> NonTrivialSmallSort for InsertionSmallSort<S, N> {}

// ---------------------------------------------------------------------------

/// Optimal sorting network for subarrays of length ≤ 8.
///
/// Uses 19 compare-and-swap operations (optimal) when len == 8.
/// Falls back to insertion sort for smaller sizes.
pub struct NetworkSmallSort;
combo_codegen::component!(SmallSort, NetworkSmallSort, "network: 8");
combo_codegen::component!(NonTrivialSmallSort, NetworkSmallSort, "network: 8");
register_small_sort!(register_network_8, NetworkSmallSort, "network: 8");

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
register_small_sort!(register_network_16, Network16SmallSort, "network: 16");

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
/// sub-arrays unsorted. After the full recursion the caller invokes
/// [`Self::final_pass`], which sweeps the whole array once and finishes
/// the sort in `O(N · THRESHOLD)`.
pub trait DeferredSmallSort {
    const THRESHOLD: usize;
    fn final_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U);
}

/// Deferred insertion: quicksort stops at sub-arrays of length ≤ N and
/// lets a single final insertion-sort pass (using strategy `S`) clean up.
pub struct DeferredInsertion<S: InsertionStrategy, const N: usize>(PhantomData<S>);
combo_codegen::component!(DeferredSmallSort, DeferredInsertion<LinearInsertion, 16>, "deferred insertion: 16");
combo_codegen::component!(DeferredSmallSort, DeferredInsertion<LinearInsertion, 32>, "deferred insertion: 32");
combo_codegen::component!(DeferredSmallSort, DeferredInsertion<BinaryInsertion, 16>, "deferred binary insertion: 16");
combo_codegen::component!(DeferredSmallSort, DeferredInsertion<BinaryInsertion, 32>, "deferred binary insertion: 32");

impl<S: InsertionStrategy, const N: usize> DeferredSmallSort for DeferredInsertion<S, N> {
    const THRESHOLD: usize = N;
    #[inline(always)]
    fn final_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let _ = insertion_sort_with::<S, _, _>(arr, logger);
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// Small sorts' `HasTimeBounds` reflects the algorithm's *intrinsic*
// complexity (assuming N is the input length). When a small sort is
// used as the bounded leaf of an outer sort (e.g. QuickSort), the outer
// sort's composition treats the slot as `O(1)` because `THRESHOLD` is
// a compile-time constant — the per-component declaration here stays
// truthful to what the algorithm is, not how it's used.
//
// Space is `O(1)` for every variant: insertion sorts run in-place, the
// hardware networks unroll into compare-and-swap sequences with no aux.

impl HasTimeBounds for NoSmallSort {
    // Never invoked (THRESHOLD = 0). Pick a value that won't pollute
    // composition; CONST keeps `Complexity::product(_, CONST) = _`.
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for NoSmallSort { const SPACE: Complexity = Complexity::CONST; }
impl HasStability for NoSmallSort { const STABLE: bool = true; }

impl HasTimeBounds for Size1SmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Size1SmallSort { const SPACE: Complexity = Complexity::CONST; }
impl HasStability for Size1SmallSort { const STABLE: bool = true; }

impl HasTimeBounds for Size2SmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Size2SmallSort { const SPACE: Complexity = Complexity::CONST; }
impl HasStability for Size2SmallSort { const STABLE: bool = true; }

// Insertion sort: O(N²) swaps in the worst case, O(N) compares on a
// pre-sorted input (best case). Stable regardless of insertion strategy
// — both linear and binary preserve original order of equal keys.
impl<S: InsertionStrategy, const N: usize> HasTimeBounds for InsertionSmallSort<S, N> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl<S: InsertionStrategy, const N: usize> HasSpace for InsertionSmallSort<S, N> {
    const SPACE: Complexity = Complexity::CONST;
}
impl<S: InsertionStrategy, const N: usize> HasStability for InsertionSmallSort<S, N> {
    const STABLE: bool = true;
}

// Sorting networks: bounded-size compare-and-swap circuits. Whether
// stable depends on the network; Batcher's odd-even and the optimal-8
// networks aren't stable in general.
impl HasTimeBounds for NetworkSmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for NetworkSmallSort { const SPACE: Complexity = Complexity::CONST; }
impl HasStability for NetworkSmallSort { const STABLE: bool = false; }

impl HasTimeBounds for Network16SmallSort {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Network16SmallSort { const SPACE: Complexity = Complexity::CONST; }
impl HasStability for Network16SmallSort { const STABLE: bool = false; }
