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

use super::compare::Compare;
use super::deep_heapify::DeepHeapify;
use super::heap::Heap;
use super::heap_partition::HeapPartition;
use crate::heap::HeapLayout;
use array_vis_bench_traits::{DualPivotSelector, PivotSelector};
use sort_logger::SortLogger;

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

// ── Variant 4: Dual-pivot stack partial-quicksort with culling ───────────────

/// Same iterative culling skeleton as `StackPartialQuickDeepHeapify`, but
/// each pop drops three sub-ranges instead of two by partitioning around
/// two pivots (Yaroslavskiy) in heap-logical coordinates. Wider fan-out
/// per pass = more aggressive culling against the layer-boundary set, so
/// fewer partition passes are needed to reach all boundaries.
///
/// Parametrised only over the [`DualPivotSelector`] — the partition itself
/// is fixed to Yaroslavskiy's three-region scan, since (unlike the
/// single-pivot path) it has no Lomuto/Hoare/Block siblings to swap in.
pub struct StackDualPivotPartialQuickDeepHeapify<DPS: DualPivotSelector>(PhantomData<DPS>);
impl<DPS: DualPivotSelector> DeepHeapify for StackDualPivotPartialQuickDeepHeapify<DPS> {
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
            if hi - lo == 2 {
                // Order the two elements per Compare so the rootward one
                // sits at the smaller logical index.
                let lo_phys = H::phys(lo, n);
                let hi_phys = H::phys(hi - 1, n);
                if !<H::Compare as Compare>::comes_first_or_eq(logger, arr, lo_phys, hi_phys) {
                    logger.swap(arr, lo_phys, hi_phys);
                }
                continue;
            }
            let (p1_logical, p2_logical) =
                select_logical_dual_pivot::<T, U, H, DPS>(arr, n, lo, hi, logger);
            let (lt, gt) = dual_pivot_partition_logical::<T, U, H>(
                arr, n, lo, hi, p1_logical, p2_logical, logger,
            );
            // LIFO: push right-most first so the left-most sub-range is
            // processed next, matching the natural recursive order.
            if gt + 1 < hi {
                stack.push((gt + 1, hi));
            }
            if lt + 1 < gt {
                stack.push((lt + 1, gt));
            }
            if lo < lt {
                stack.push((lo, lt));
            }
        }
    }
}

/// Convert physical-slice indices returned by a [`DualPivotSelector`] over
/// `arr[phys_start..phys_end]` back into logical indices in `[lo, hi)`.
fn select_logical_dual_pivot<T, U, H, DPS>(
    arr: &mut [T],
    n: usize,
    lo: usize,
    hi: usize,
    logger: &mut U,
) -> (usize, usize)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    H: HeapLayout,
    DPS: DualPivotSelector,
{
    let (phys_start, phys_end) = logical_to_physical_range::<H>(lo, hi, n);
    let (p1_in_slice, p2_in_slice) = DPS::select(&arr[phys_start..phys_end], logger);
    let p1_logical = H::phys(phys_start + p1_in_slice, n);
    let p2_logical = H::phys(phys_start + p2_in_slice, n);
    (p1_logical, p2_logical)
}

/// Yaroslavskiy three-region partition in heap-logical coordinates.
///
/// Places pivot `p1` at logical `lo` and pivot `p2` at logical `hi - 1`,
/// normalised so `arr[lo]` is more rootward than `arr[hi - 1]` per
/// `H::Compare`. Returns `(lt, gt)` such that after the call:
///
/// - `arr_logical[lo..lt]`        — more rootward than the first pivot
/// - `arr_logical[lt]`            — first pivot (placed)
/// - `arr_logical[lt + 1..gt]`    — between the two pivots
/// - `arr_logical[gt]`            — second pivot (placed)
/// - `arr_logical[gt + 1..hi]`    — less rootward than the second pivot
///
/// All comparisons go through `H::Compare` so the routine works for any
/// direction; all physical access goes through `H::phys` so visualiser
/// events land at the real positions.
fn dual_pivot_partition_logical<T, U, H>(
    arr: &mut [T],
    n: usize,
    lo: usize,
    hi: usize,
    p1_logical: usize,
    p2_logical: usize,
    logger: &mut U,
) -> (usize, usize)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    H: HeapLayout,
{
    let last = hi - 1;
    let lo_phys = H::phys(lo, n);
    let last_phys = H::phys(last, n);

    // Place p1 at logical `lo`, then p2 at logical `last` — with the same
    // collision-fixup the value-pivoted dual-pivot quicksort uses.
    logger.swap(arr, H::phys(p1_logical, n), lo_phys);
    let p2_logical = if p2_logical == p1_logical {
        lo
    } else if p2_logical == lo {
        p1_logical
    } else {
        p2_logical
    };
    logger.swap(arr, H::phys(p2_logical, n), last_phys);

    // Ensure rootward order: arr[lo] should be more-or-equally rootward
    // than arr[last]. If not, swap the two pivots.
    if !<H::Compare as Compare>::comes_first_or_eq(logger, arr, lo_phys, last_phys) {
        logger.swap(arr, lo_phys, last_phys);
    }

    let mut lt = lo + 1;
    let mut i = lo + 1;
    let mut gt = last - 1;

    // Pivots stay anchored at logical `lo` and `last` for the whole scan,
    // so we keep comparing against `lo_phys` / `last_phys` directly.
    while i <= gt {
        let i_phys = H::phys(i, n);
        if <H::Compare as Compare>::comes_first(logger, arr, i_phys, lo_phys) {
            logger.swap(arr, i_phys, H::phys(lt, n));
            lt += 1;
            i += 1;
        } else if <H::Compare as Compare>::comes_first(logger, arr, last_phys, i_phys) {
            while i < gt
                && <H::Compare as Compare>::comes_first(logger, arr, last_phys, H::phys(gt, n))
            {
                gt -= 1;
            }
            logger.swap(arr, i_phys, H::phys(gt, n));
            if gt == lo {
                break;
            }
            gt -= 1;
            let i_phys_after = H::phys(i, n);
            if <H::Compare as Compare>::comes_first(logger, arr, i_phys_after, lo_phys) {
                logger.swap(arr, i_phys_after, H::phys(lt, n));
                lt += 1;
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    lt -= 1;
    gt += 1;
    logger.swap(arr, lo_phys, H::phys(lt, n));
    logger.swap(arr, last_phys, H::phys(gt, n));

    (lt, gt)
}
