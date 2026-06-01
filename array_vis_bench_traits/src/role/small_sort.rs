//! `SmallSort` and its companion traits.
//!
//! Leaf crates (`small_sort_basic`, `small_sort_insertion`, …) implement
//! these and live in their own tiny crates. The `register_small_sort!`
//! macro in `array_vis_bench` wires each registered variant into the
//! standalone-algorithm registry.

use std::marker::PhantomData;

use sort_logger::SortLogger;

// ── InsertionStrategy ────────────────────────────────────────────────────────

/// Strategy for placing one element of a sorted prefix.
///
/// Given `arr[..i]` already sorted and `arr[i]` the new element, an
/// implementor moves `arr[i]` to its correct position within
/// `arr[..=i]`. Returns `true` if the array was mutated.
pub trait InsertionStrategy {
    fn insert_one<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        logger: &mut U,
    ) -> bool;
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

/// Insertion sort with a bounded scan-back window of size `K`.
///
/// Caller contract: every element is at most `K` positions from its
/// final sorted position. Each element is then placed by running
/// `S::insert_one` on the slice `arr[i.saturating_sub(K)..=i]`, so the
/// inner loop never scans past `K` predecessors regardless of input
/// size. Total cost is `O(n · K)`.
///
/// Used by `DeferredInsertion` to clean up after a deferred quicksort
/// whose recursion stopped at chunks of size ≤ `K`.
#[inline(always)]
pub fn windowed_insertion_sort_with<S: InsertionStrategy, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    k: usize,
    logger: &mut U,
) -> bool {
    let mut mutated = false;
    for i in 1..arr.len() {
        let lo = i.saturating_sub(k);
        let sub = &mut arr[lo..=i];
        let local_i = sub.len() - 1;
        mutated |= S::insert_one(sub, local_i, logger);
    }
    mutated
}

// ── SmallSort ────────────────────────────────────────────────────────────────

/// Strategy for sorting small sub-arrays before or during a merge /
/// quick / shell pass.
///
/// # Contract
///
/// Implementors expose a compile-time `THRESHOLD`. If 0 the small sort
/// is never triggered; otherwise [`Self::sort`] **must produce a fully
/// sorted output for any `arr.len() <= THRESHOLD`** — including 0, 1, 2,
/// … up to and including `THRESHOLD`. Callers may pass any length within
/// that range (some pad/clamp at boundaries), so a network-style impl
/// that only sorts at exactly its peak size must fall back to insertion
/// sort (or equivalent) for smaller inputs.
///
/// `sort` returns `true` if the array was mutated (any swap happened),
/// `false` if it was already sorted. All impls are `#[inline(always)]`
/// so a caller that discards the bool gets the change-tracking
/// dead-code-eliminated by the compiler.
pub trait SmallSort {
    /// Subarray length at or below which this strategy is invoked
    /// (0 = never).
    const THRESHOLD: usize;

    /// Sort `arr` in-place. Caller guarantees `arr.len() <=
    /// Self::THRESHOLD`; implementor guarantees correct sorted output
    /// for **any** such length.
    ///
    /// Returns `true` if the array was mutated, `false` if it was
    /// already sorted.
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool;
}

/// Subtrait of [`SmallSort`] for variants whose threshold is strictly
/// above 1 — i.e. they actually do sorting work for arrays larger than
/// a single element. Use this bound on sorts whose algorithm relies on
/// the small sort to make meaningful progress (e.g. block-level
/// odd-even).
pub trait NonTrivialSmallSort: SmallSort {}

// ── SetSizeSmallSort ─────────────────────────────────────────────────────────

/// Strategy for sorting a sub-array of a *fixed* compile-time size `N`.
///
/// Unlike [`SmallSort`] — which guarantees correctness for any length
/// up to its threshold — implementors of `SetSizeSmallSort<N>` only
/// handle arrays of exactly `N` elements. The size is enforced at the
/// type level via `&mut [T; N]`.
pub trait SetSizeSmallSort<const N: usize> {
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T; N], logger: &mut U);
}

/// Adapter exposing any [`SmallSort`] as a [`SetSizeSmallSort<N>`] for
/// every `N <= S::THRESHOLD`. The bound is enforced at compile time via
/// an inline `const` assertion. The mutation bool returned by the
/// underlying sort is discarded — fixed-size callers don't need it.
pub struct SmallSortAdapter<S: SmallSort>(PhantomData<S>);

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

// ── DeferredSmallSort ────────────────────────────────────────────────────────

/// Marks a threshold at which quicksort stops recursing, leaving small
/// sub-arrays unsorted. After the full recursion the caller invokes
/// [`Self::final_pass`], which sweeps the whole array once and finishes
/// the sort in `O(N · THRESHOLD)`.
pub trait DeferredSmallSort {
    const THRESHOLD: usize;
    fn final_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U);
}
