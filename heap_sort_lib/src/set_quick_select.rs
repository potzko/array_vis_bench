//! `SetQuickSelect` — multi-position quickselect contract for heap building.
//!
//! Wraps the "place the value that ranks at each heap layer boundary" job
//! that the old `QuickDeepHeapify` variants do, but expressed in terms of
//! the [`QuickSelect`] trait so the partition + pivot choice can be
//! reused from the rest of the catalog. Three implementations:
//!
//! 1. [`StackSet`] — pass-through to the hand-optimised
//!    [`StackPartialQuickDeepHeapify`](super::quick_deep_heapify::StackPartialQuickDeepHeapify),
//!    which has its own iterative stack-based culling that doesn't decompose
//!    naturally into per-boundary `QuickSelect::select` calls.
//! 2. [`SequentialSet<QS>`] — sequential quickselect at each boundary,
//!    descending layer order, tightening the upper bound after each.
//! 3. [`RecursiveSet<QS>`] — recursive partial quicksort using
//!    `QuickSelect`: split at the median boundary, recurse into the half
//!    that still contains a layer boundary.
//!
//! ## Direction handling
//!
//! `QuickSelect` orders by `Ord` ascending — its output puts the smaller
//! values at low physical indices. That matches a `MinForward` /
//! `MaxReverse` heap directly. For `MaxForward` and `MinReverse` the
//! rootward direction is the opposite end, so after the per-boundary work
//! we reverse the array once. The reverse is logged so the visualiser
//! sees each swap.
//!
//! Each impl also impls [`DeepHeapify`] (the trait `NaryHeapSort` and
//! friends consume), so a `SetQuickSelect` impl drops into any existing
//! heap-build slot.

use std::marker::PhantomData;

use array_vis_bench_traits::QuickSelect;
use sort_logger::SortLogger;

use crate::compare::Compare;
use crate::deep_heapify::DeepHeapify;
use crate::heap::{Heap, HeapLayout};
use crate::heap_partition::HeapPartition;
use crate::quick_deep_heapify::StackPartialQuickDeepHeapify;
use array_vis_bench_traits::PivotSelector;

// ── SetQuickSelect ───────────────────────────────────────────────────────────

pub trait SetQuickSelect {
    fn set_quick_select<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    );
}

/// Per-`Compare` "natural" build: `Compare=Min` produces a MinForward
/// layout, `Compare=Max` produces a MaxReverse layout. `reverse_slice`
/// then bridges within each Compare's pair to the actual Layout.
///
/// Returns `true` iff the algorithm's natural Layout matches `H`'s
/// actual Layout — i.e. no final reverse needed.
#[inline(always)]
fn natural_layout_matches<H: Heap>(n: usize) -> bool {
    let rootward_is_smaller_ord =
        <<H as HeapLayout>::Compare as Compare>::ROOTWARD_IS_SMALLER_ORD;
    let rootward_is_low_phys = H::phys(0, n) == 0;
    rootward_is_smaller_ord == rootward_is_low_phys
}

/// `true` if the per-`Compare` natural build for `H` is MinForward
/// (i.e. `Compare=Min`). Otherwise the natural build is MaxReverse.
#[inline(always)]
fn build_min_forward<H: Heap>() -> bool {
    <<H as HeapLayout>::Compare as Compare>::ROOTWARD_IS_SMALLER_ORD
}

/// Reverse `arr` via logged swaps. The visualiser sees each swap.
fn reverse_slice<T, U>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
{
    let n = arr.len();
    if n < 2 {
        return;
    }
    let mut i = 0;
    let mut j = n - 1;
    while i < j {
        logger.swap(arr, i, j);
        i += 1;
        j -= 1;
    }
}

// ── 1) StackSet (hand-optimised wrapper) ─────────────────────────────────────

/// Pass-through to [`StackPartialQuickDeepHeapify`] — its iterative culling
/// stack isn't expressible as a sequence of single-target `QuickSelect`
/// calls, so it keeps its bespoke shape and just re-exposes itself under
/// the [`SetQuickSelect`] surface for catalog symmetry.
pub struct StackSet<HP: HeapPartition, V: PivotSelector> {
    _phantom: PhantomData<(HP, V)>,
}

impl<HP: HeapPartition, V: PivotSelector> SetQuickSelect for StackSet<HP, V> {
    #[inline(always)]
    fn set_quick_select<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        <StackPartialQuickDeepHeapify<HP, V> as DeepHeapify>::deep_heapify::<H, T, U>(arr, logger);
    }
}

impl<HP: HeapPartition, V: PivotSelector> DeepHeapify for StackSet<HP, V> {
    #[inline(always)]
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        <Self as SetQuickSelect>::set_quick_select::<H, T, U>(arr, logger);
    }
}

// ── 2) SequentialSet<QS> ─────────────────────────────────────────────────────

/// At each heap layer boundary `B` (logical, descending order), call
/// `QS::select(arr[..upper], B)` and tighten `upper` to `B`. After
/// `floor(log_A n)` calls every layer boundary is placed; the array is in
/// ascending order at the partition points. For heaps whose rootward
/// direction doesn't match ascending (`MaxForward`, `MinReverse`), the
/// final `reverse_slice` flips it into the correct shape.
pub struct SequentialSet<QS: QuickSelect> {
    _phantom: PhantomData<QS>,
}

impl<QS: QuickSelect> SetQuickSelect for SequentialSet<QS> {
    fn set_quick_select<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let boundaries = H::layer_boundaries(n);
        if build_min_forward::<H>() {
            // MinForward natural build: place B smallest at [0..B] per
            // boundary, narrowing `upper` toward 0.
            let mut upper = n;
            for &b in boundaries.iter().rev() {
                if b >= upper || upper < 2 {
                    continue;
                }
                QS::select(&mut arr[..upper], logger, b);
                upper = b;
            }
        } else {
            // MaxReverse natural build: place B largest at [n-B..n] per
            // boundary, growing `start` toward n. Same `QS::select` call,
            // just targeted at the high end of the active slice.
            let mut start = 0;
            for &b in boundaries.iter().rev() {
                let active_len = n - start;
                if b >= active_len || active_len < 2 {
                    continue;
                }
                let target = active_len - b - 1;
                QS::select(&mut arr[start..], logger, target);
                start = n - b;
            }
        }
        if !natural_layout_matches::<H>(n) {
            reverse_slice(arr, logger);
        }
    }
}

impl<QS: QuickSelect> DeepHeapify for SequentialSet<QS> {
    #[inline(always)]
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        <Self as SetQuickSelect>::set_quick_select::<H, T, U>(arr, logger);
    }
}

// ── 3) RecursiveSet<QS> ──────────────────────────────────────────────────────

/// Recursive partial quicksort using `QuickSelect::select`: at each call
/// pick the median layer boundary lying in the active range, place it via
/// one `QS::select`, recurse into each half *only if* the half still
/// contains a layer boundary (cull otherwise). Same asymptotic cost as
/// `SequentialSet` but the recursion shape exposes a different
/// visualiser trace.
pub struct RecursiveSet<QS: QuickSelect> {
    _phantom: PhantomData<QS>,
}

impl<QS: QuickSelect> SetQuickSelect for RecursiveSet<QS> {
    fn set_quick_select<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let boundaries = H::layer_boundaries(n);
        if build_min_forward::<H>() {
            recurse_min_forward::<QS, T, U>(arr, 0, n, &boundaries, logger);
        } else {
            recurse_max_reverse::<QS, T, U>(arr, n, 0, n, &boundaries, logger);
        }
        if !natural_layout_matches::<H>(n) {
            reverse_slice(arr, logger);
        }
    }
}

impl<QS: QuickSelect> DeepHeapify for RecursiveSet<QS> {
    #[inline(always)]
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        <Self as SetQuickSelect>::set_quick_select::<H, T, U>(arr, logger);
    }
}

/// MinForward recursion: pick the median *logical* boundary B in
/// `[lo, hi)`, `QS::select` to place the B-th-smallest at arr[B], then
/// recurse on each half that still contains a boundary.
fn recurse_min_forward<QS: QuickSelect, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    boundaries: &[usize],
    logger: &mut U,
) {
    if hi - lo < 2 {
        return;
    }
    // `<= lo` (not `< lo`) so a boundary AT lo is skipped — the parent call
    // already placed that value, and re-picking it would feed the same
    // `(b, hi)` slice back into the recursion (infinite loop). Mirrors the
    // `<= log_lo` predicate in `recurse_max_reverse`.
    let t_lo = boundaries.partition_point(|&t| t <= lo);
    let t_hi = boundaries.partition_point(|&t| t < hi);
    if t_lo == t_hi {
        return;
    }
    let mid_idx = t_lo + (t_hi - t_lo) / 2;
    let b = boundaries[mid_idx];
    QS::select(&mut arr[lo..hi], logger, b - lo);
    recurse_min_forward::<QS, T, U>(arr, lo, b, boundaries, logger);
    recurse_min_forward::<QS, T, U>(arr, b, hi, boundaries, logger);
}

/// MaxReverse recursion: mirror of `recurse_min_forward`. For each
/// logical boundary B the physical split is at `n - B` (rootward layer
/// is at the high end). The smaller-Ord side ends up at the low end of
/// the slice (deeper layers); the larger-Ord side at the high end
/// (rootward layer).
fn recurse_max_reverse<QS: QuickSelect, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    n: usize,
    lo: usize,
    hi: usize,
    boundaries: &[usize],
    logger: &mut U,
) {
    if hi - lo < 2 {
        return;
    }
    // Logical boundaries whose physical split `n - B` falls in `(lo, hi)`,
    // i.e. logical `B` in `(n - hi, n - lo)`.
    let log_lo = n - hi;
    let log_hi = n - lo;
    let t_lo = boundaries.partition_point(|&t| t <= log_lo);
    let t_hi = boundaries.partition_point(|&t| t < log_hi);
    if t_lo == t_hi {
        return;
    }
    let mid_idx = t_lo + (t_hi - t_lo) / 2;
    let b = boundaries[mid_idx];
    let phys_split = n - b;
    // Place the value such that arr[lo..phys_split] = (phys_split - lo)
    // smallest of the slice, arr[phys_split..hi] = (hi - phys_split) largest.
    let target_in_slice = phys_split - 1 - lo;
    QS::select(&mut arr[lo..hi], logger, target_in_slice);
    recurse_max_reverse::<QS, T, U>(arr, n, lo, phys_split, boundaries, logger);
    recurse_max_reverse::<QS, T, U>(arr, n, phys_split, hi, boundaries, logger);
}
