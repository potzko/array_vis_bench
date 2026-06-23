use std::marker::PhantomData;
use std::ops::Range;

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds, PivotQuality};
use array_vis_bench_traits::SmallSort;
use sort_logger::SortLogger;

use array_vis_bench_traits::{
    with_partition_scratch, PartitionScheme, PartitionVisitor, PivotInput,
};

use crate::pivoted::Pivoted;

/// Generic quicksort. `P` decides how each partition step works
/// (single- or dual-pivot — see [`PartitionScheme::N_PIVOTS`]); `V`
/// decides where the pivots come from for each call (single- or
/// dual-pivot — see [`PivotInput::N`]). The two arities must agree at
/// the type level: `P::N_PIVOTS == V::N`.
///
/// Single-pivot variants are written as `QuickSort<LeftLeftPartition, FirstElement,
/// NoSmallSort>`; dual-pivot as
/// `QuickSort<DualPivotPartition, CombinedSelector<First, Middle>, NoSmallSort>`.
/// The old `DualPivotQuickSort` is gone — both shapes route through one
/// recursive driver below.
pub struct QuickSort<P: PartitionScheme, V: PivotInput, SS: SmallSort>(
    PhantomData<(P, V, SS)>,
);

impl<P: PartitionScheme, V: PivotInput, SS: SmallSort> QuickSort<P, V, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // Pivot ownership now lives in the partition: fuse `P` + `V` into a
        // self-pivoting `Pivoted<P, V>` and run it through the single
        // pivot-free driver. `QuickSort<P, V, SS>` is kept as a thin
        // compatibility shim (the catalog still emits this type name) — it
        // owns no recursion of its own anymore.
        PivotlessQuickSort::<Pivoted<P, V>, SS>::sort(arr, logger)
    }
}

/// Stack-resident visitor: collects up to 4 unsorted ranges the
/// partition emits per call. 4 is the upper bound across every
/// `PartitionScheme` impl in the workspace (3 for DualPivotPartition
/// dual-pivot, 2 for single-pivot). Tagged `#[inline(always)]` so the
/// visitor dispatch lowers to direct stack writes after monomorphisation.
struct QuickSortVisitor {
    ranges: [Range<usize>; 4],
    n: u8,
}

impl QuickSortVisitor {
    #[inline(always)]
    fn new() -> Self {
        Self { ranges: [0..0, 0..0, 0..0, 0..0], n: 0 }
    }
}

impl PartitionVisitor for QuickSortVisitor {
    #[inline(always)]
    fn unsorted(&mut self, r: Range<usize>) {
        // Safety: trait contract caps partition emit at 4 ranges per call.
        unsafe { *self.ranges.get_unchecked_mut(self.n as usize) = r };
        self.n += 1;
    }
}

fn pivotless_recursive<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    SS: SmallSort,
>(
    arr: &mut [T],
    scratch: &mut [usize],
    logger: &mut U,
) {
    if SS::THRESHOLD > 0 && arr.len() <= SS::THRESHOLD {
        SS::sort(arr, logger);
        return;
    }
    if arr.len() < 2 {
        return;
    }
    // The partition is self-contained: it picks its own pivots (if any)
    // and announces the unsorted regions. The driver supplies none.
    let mut visitor = QuickSortVisitor::new();
    P::partition::<T, U, _>(arr, logger, &[], scratch, &mut visitor);
    let n = visitor.n as usize;
    let mut i = 0;
    while i < n {
        let r = visitor.ranges[i].clone();
        pivotless_recursive::<T, U, P, SS>(&mut arr[r], scratch, logger);
        i += 1;
    }
}

/// Pivotless quicksort: the partition decides its own split without
/// consulting pivot *values* (e.g. [`HeapExtract`](https://docs.rs/quick_heap_sort_lib)
/// splits at the midpoint). This is the structural counterpart to
/// [`QuickSort`] — same recursive driver, but the sort carries **no pivot
/// vocabulary at all** instead of papering over the absence with `NoPivot`.
///
/// `P` must be a zero-pivot scheme (`P::N_PIVOTS == 0`); it receives an
/// empty `pivots` slice every call. Internally this routes through the
/// shared [`QuickSort`] driver with the zero-pivot [`NoPivot`] input, so
/// there is a single recursion to maintain — but the public type
/// `PivotlessQuickSort<P, SS>` exposes no pivot parameter, which is the
/// whole point: pivot-based and pivotless quicksorts are now distinct
/// types rather than one type with a degenerate pivot slot.
pub struct PivotlessQuickSort<P: PartitionScheme, SS: SmallSort>(PhantomData<(P, SS)>);

impl<P: PartitionScheme, SS: SmallSort> PivotlessQuickSort<P, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // The single quicksort recursion. Pre-allocate the partition's
        // scratch buffer once and thread it through every call (as the
        // merge sorts allocate their aux array once).
        with_partition_scratch::<P, T, U, _>(logger, |logger, scratch| {
            pivotless_recursive::<T, U, P, SS>(arr, scratch, logger);
        });
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// Per-level work: partition + pivot selection. The small-sort slot
// contributes O(1) to QuickSort's overall complexity because it only
// runs on bounded-size slices (`arr.len() <= SS::THRESHOLD`) — so the
// composition ignores `SS::WORST` and uses `Complexity::CONST` for that
// slot, even when the SS algorithm is intrinsically O(N²).
//
// Worst case: degenerate pivots collapse to O(N) recursion depth →
//             O(N · per-level). Median-of-medians keeps it at O(log N).
// Best / average: O(log N) depth assuming balanced partitions.

impl<P, V, SS> HasTimeBounds for QuickSort<P, V, SS>
where
    P: PartitionScheme + HasTimeBounds,
    V: PivotInput + HasTimeBounds + PivotQuality,
    SS: SmallSort,
{
    /// Recursion depth × per-level (partition + pivot) × small-sort (O(1)).
    /// Depth is O(N) if the pivot can degenerate, else O(log N).
    const WORST: Complexity = Complexity::product(
        if V::DEGENERATES { Complexity::N1 } else { Complexity::LOG_N },
        Complexity::sum(P::WORST, V::WORST),
    );
    /// Balanced split → O(log N) depth.
    const BEST: Complexity = Complexity::product(
        Complexity::LOG_N,
        Complexity::sum(P::BEST, V::BEST),
    );
    const AVERAGE: Complexity = Complexity::product(
        Complexity::LOG_N,
        Complexity::sum(P::AVERAGE, V::AVERAGE),
    );
}

impl<P, V, SS> HasSpace for QuickSort<P, V, SS>
where
    P: PartitionScheme + HasSpace,
    V: PivotInput + HasSpace,
    SS: SmallSort + HasSpace,
{
    /// Recursion adds O(log N) stack baseline; take the max with each
    /// component's own aux-space contribution.
    const SPACE: Complexity = Complexity::sum(
        Complexity::LOG_N,
        Complexity::sum(P::SPACE, Complexity::sum(V::SPACE, SS::SPACE)),
    );
}

impl<P, V, SS> HasStability for QuickSort<P, V, SS>
where
    P: PartitionScheme + HasStability,
    V: PivotInput + HasStability,
    SS: SmallSort + HasStability,
{
    const STABLE: bool = P::STABLE && V::STABLE && SS::STABLE;
}

// ── Composable annotations for the pivotless driver ──────────────────
//
// The driver reads everything off the partition `P` alone — it is now
// pivot-agnostic. Worst-case recursion depth is O(N) if the partition can
// degenerate (a `Pivoted<…>` wrapping a degenerate selector) and O(log N)
// otherwise (a self-balancing splitter like `HeapExtract`, whose
// `PivotQuality::DEGENERATES` is `false`). Per-level cost is the
// partition's; the small sort runs only on bounded leaves → O(1).

impl<P, SS> HasTimeBounds for PivotlessQuickSort<P, SS>
where
    P: PartitionScheme + HasTimeBounds + PivotQuality,
    SS: SmallSort,
{
    const WORST: Complexity = Complexity::product(
        if P::DEGENERATES { Complexity::N1 } else { Complexity::LOG_N },
        P::WORST,
    );
    const BEST: Complexity = Complexity::product(Complexity::LOG_N, P::BEST);
    const AVERAGE: Complexity = Complexity::product(Complexity::LOG_N, P::AVERAGE);
}

impl<P, SS> HasSpace for PivotlessQuickSort<P, SS>
where
    P: PartitionScheme + HasSpace,
    SS: SmallSort + HasSpace,
{
    const SPACE: Complexity = Complexity::sum(
        Complexity::LOG_N,
        Complexity::sum(P::SPACE, SS::SPACE),
    );
}

impl<P, SS> HasStability for PivotlessQuickSort<P, SS>
where
    P: PartitionScheme + HasStability,
    SS: SmallSort + HasStability,
{
    const STABLE: bool = P::STABLE && SS::STABLE;
}
