//! Build-the-heap-via-quickselect strategies for [`DeepHeapify`].
//!
//! Instead of bottom-up sift-down (the textbook `Iterative` build), these
//! variants establish layer-boundary ranks via quickselect:
//!
//! Layer `k` of an A-ary heap occupies logical indices `[B_{k-1}, B_k)`
//! with `B_0 = 1`, `B_k = B_{k-1} + A^k`. Quickselecting at logical
//! `B_{k-1}` puts the top `B_{k-1}` most-rootward values (per `H::Compare`)
//! in the array's logical-low end and pushes everything else into the
//! logical-high region — which means every node in layer `< k` ends up
//! "more rootward" than every node in layer `≥ k`. Repeating this for all
//! layer boundaries yields a valid heap: the heap predicate at every
//! parent holds because the parent lives in a strictly-more-rootward layer
//! than its children.
//!
//! Three variants differ only in *how* the partition work is scheduled
//! across boundaries:
//!
//! - [`SequentialQuickDeepHeapify`] — one quickselect per boundary,
//!   bottom-up; each runs on the shrinking logical prefix `[0..upper_bound)`.
//! - [`RecursivePartialQuickDeepHeapify`] — recursive partial quicksort:
//!   partition once, recurse into both halves, cull frames whose range
//!   contains no boundary index.
//! - [`StackPartialQuickDeepHeapify`] — same algorithm as the recursive
//!   variant, control-flow-flattened with an explicit `(lo, hi)` stack
//!   (analog of `cyclent_sort_stack_optimized`).
//!
//! All variants use the heap-aware [`HeapPartition`] (direction-driven by
//! `H::Compare`) so they work for both `MaxForward` and `MinReverse`
//! heaps; pivot indices come from the standard [`PivotSelector`] applied
//! to the appropriate physical sub-slice and mapped back through
//! `H::phys`.

use std::marker::PhantomData;

use super::deep_heapify::DeepHeapify;
use super::heap::Heap;
use super::heap_partition::HeapPartition;
use crate::sorts::heap_sort::heap::HeapLayout;
use crate::sorts::quick_sorts::pivot_selectors::PivotSelector;
use crate::traits::log_traits::SortLogger;

// ── Shared helpers ───────────────────────────────────────────────────────────

/// Physical sub-slice (start, end) corresponding to logical `[lo..hi)`.
/// Works for both `Forward` (start=lo, end=hi) and `Reverse` (start=n-hi,
/// end=n-lo) layouts via `H::phys`.
fn logical_to_physical_range<H: HeapLayout>(lo: usize, hi: usize, n: usize) -> (usize, usize) {
    let a = H::phys(lo, n);
    let b = H::phys(hi - 1, n);
    (a.min(b), a.max(b) + 1)
}

/// Pick a pivot via `V` (a slice-based selector) over the physical
/// sub-slice for logical `[lo..hi)`; return the LOGICAL index of the chosen
/// pivot. `V`'s value-based selection (median-of-three etc.) is still
/// meaningful regardless of layout — it picks a representative value, and
/// the *value* is what governs partition quality.
fn select_logical_pivot<T, U, H, V>(
    arr: &mut [T],
    n: usize,
    lo: usize,
    hi: usize,
    logger: &mut U,
) -> usize
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    H: HeapLayout,
    V: PivotSelector,
{
    let (phys_start, phys_end) = logical_to_physical_range::<H>(lo, hi, n);
    let p_in_slice = V::select(&arr[phys_start..phys_end], logger);
    H::phys(phys_start + p_in_slice, n)
}

/// Logical-coordinate quickselect: arrange so `arr_logical[target]` ends
/// up holding the `(target+1)`-th most-rootward value and arr_logical[..target]
/// is "more rootward" than the value at `target`.
fn quickselect_logical<T, U, H, HP, V>(
    arr: &mut [T],
    n: usize,
    upper_bound: usize,
    target: usize,
    logger: &mut U,
) where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    H: Heap,
    HP: HeapPartition,
    V: PivotSelector,
{
    let mut lo = 0usize;
    let mut hi = upper_bound;
    while hi - lo >= 2 {
        let pivot = select_logical_pivot::<T, U, H, V>(arr, n, lo, hi, logger);
        let (left_end, right_start) = HP::partition::<T, U, H>(arr, n, lo, hi, pivot, logger);
        if target < left_end {
            hi = left_end;
        } else if target >= right_start {
            lo = right_start;
        } else {
            return;
        }
    }
}

// ── Variant 1: Sequential bottom-up ──────────────────────────────────────────

/// One quickselect per layer boundary, bottom-up. Each runs on the
/// logical prefix `[0..upper_bound)` and tightens `upper_bound` to the
/// boundary it just placed.
pub struct SequentialQuickDeepHeapify<HP: HeapPartition, V: PivotSelector>(PhantomData<(HP, V)>);
combo_codegen::component!(QuickDeepHeapify, SequentialQuickDeepHeapify, "sequential");

impl<HP: HeapPartition, V: PivotSelector> DeepHeapify for SequentialQuickDeepHeapify<HP, V> {
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let boundaries = H::layer_boundaries(n);
        let mut upper_bound = n;
        for &target in boundaries.iter().rev() {
            if target >= upper_bound || upper_bound < 2 {
                continue;
            }
            quickselect_logical::<T, U, H, HP, V>(arr, n, upper_bound, target, logger);
            upper_bound = target;
        }
    }
}

// ── Variant 2: Recursive partial-quicksort with culling ──────────────────────

/// Partition the full array once, then recurse into BOTH halves, but only
/// when the half's logical range still contains at least one layer
/// boundary. Frames whose `[lo..hi)` range is boundary-free are culled.
pub struct RecursivePartialQuickDeepHeapify<HP: HeapPartition, V: PivotSelector>(
    PhantomData<(HP, V)>,
);
combo_codegen::component!(
    QuickDeepHeapify,
    RecursivePartialQuickDeepHeapify,
    "recursive partition"
);

impl<HP: HeapPartition, V: PivotSelector> DeepHeapify for RecursivePartialQuickDeepHeapify<HP, V> {
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let boundaries = H::layer_boundaries(n);
        partial_recurse::<T, U, H, HP, V>(arr, n, 0, n, &boundaries, logger);
    }
}

fn partial_recurse<T, U, H, HP, V>(
    arr: &mut [T],
    n: usize,
    lo: usize,
    hi: usize,
    boundaries: &[usize],
    logger: &mut U,
) where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    H: Heap,
    HP: HeapPartition,
    V: PivotSelector,
{
    if hi - lo < 2 {
        return;
    }
    // Cull: if no layer boundary falls in [lo, hi) we don't need to
    // place anything here.
    let t_lo = boundaries.partition_point(|&t| t < lo);
    let t_hi = boundaries.partition_point(|&t| t < hi);
    if t_lo == t_hi {
        return;
    }
    let pivot = select_logical_pivot::<T, U, H, V>(arr, n, lo, hi, logger);
    let (left_end, right_start) = HP::partition::<T, U, H>(arr, n, lo, hi, pivot, logger);
    partial_recurse::<T, U, H, HP, V>(arr, n, lo, left_end, boundaries, logger);
    partial_recurse::<T, U, H, HP, V>(arr, n, right_start, hi, boundaries, logger);
}

// ── Variant 3: Iterative partial-quicksort with explicit stack ───────────────

/// Same partial-quicksort algorithm as `RecursivePartial`, but the
/// recursion is unrolled into an explicit `(lo, hi)` stack.
pub struct StackPartialQuickDeepHeapify<HP: HeapPartition, V: PivotSelector>(
    PhantomData<(HP, V)>,
);
combo_codegen::component!(
    QuickDeepHeapify,
    StackPartialQuickDeepHeapify,
    "stack partition"
);

impl<HP: HeapPartition, V: PivotSelector> DeepHeapify for StackPartialQuickDeepHeapify<HP, V> {
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let boundaries = H::layer_boundaries(n);
        let mut stack: Vec<(usize, usize)> = Vec::new();
        stack.push((0, n));
        while let Some((lo, hi)) = stack.pop() {
            if hi - lo < 2 {
                continue;
            }
            let t_lo = boundaries.partition_point(|&t| t < lo);
            let t_hi = boundaries.partition_point(|&t| t < hi);
            if t_lo == t_hi {
                continue;
            }
            let pivot = select_logical_pivot::<T, U, H, V>(arr, n, lo, hi, logger);
            let (left_end, right_start) = HP::partition::<T, U, H>(arr, n, lo, hi, pivot, logger);
            // LIFO: push right first so left is processed next, matching
            // the recursive variant's order.
            if right_start < hi {
                stack.push((right_start, hi));
            }
            if lo < left_end {
                stack.push((lo, left_end));
            }
        }
    }
}
