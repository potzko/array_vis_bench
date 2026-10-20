use std::marker::PhantomData;
use std::ops::Range;

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds, PivotQuality};
use array_vis_bench_traits::SmallSort;
use sort_logger::SortLogger;

use array_vis_bench_traits::{with_partition_scratch, PartitionScheme, PartitionVisitor};

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

/// The **canonical** quicksort driver: it speaks only the self-splitting
/// [`PartitionScheme`] contract (`P::N_PIVOTS == 0`) — hands the partition an
/// empty `pivots` slice and recurses on the unsorted ranges the partition
/// announces. It is **pivot-agnostic**: `P` may be a genuinely pivotless
/// splitter (`HeapExtract`, which splits at the midpoint) or a pivot-based
/// scheme made self-pivoting by wrapping it as [`Pivoted<Scheme, Sel>`](crate::pivoted::Pivoted).
/// Either way the driver carries no pivot vocabulary — that lives entirely in
/// `P` — so there is one recursion to maintain.
///
/// This is THE default quicksort recursion: the catalog emits every pivot-based
/// quicksort as `CanonicalQuickSort<Pivoted<Scheme, Sel>, SS>` directly (there is
/// no separate pivot-carrying `QuickSort` type). A
/// sentinel-eliminating driver that DOES exploit pivot/position
/// (`SentinelQuickSort`, see `docs/implementation_plans/sentinel_quicksort.md`)
/// is a separate, parked design — this driver deliberately stays bare.
pub struct CanonicalQuickSort<P: PartitionScheme, SS: SmallSort>(PhantomData<(P, SS)>);

impl<P: PartitionScheme, SS: SmallSort> CanonicalQuickSort<P, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // The single quicksort recursion. Pre-allocate the partition's
        // scratch buffer once and thread it through every call (as the
        // merge sorts allocate their aux array once).
        with_partition_scratch::<P, T, U, _>(logger, |logger, scratch| {
            pivotless_recursive::<T, U, P, SS>(arr, scratch, logger);
        });
    }
}

array_vis_bench_traits::direct_sort!(CanonicalQuickSort<P, SS>; P: PartitionScheme, SS: SmallSort);

// ── Composable annotations for the pivotless driver ──────────────────
//
// The driver reads everything off the partition `P` alone — it is now
// pivot-agnostic. Worst-case recursion depth is O(N) if the partition can
// degenerate (a `Pivoted<…>` wrapping a degenerate selector) and O(log N)
// otherwise (a self-balancing splitter like `HeapExtract`, whose
// `PivotQuality::DEGENERATES` is `false`). Per-level cost is the
// partition's; the small sort runs only on bounded leaves → O(1).

impl<P, SS> HasTimeBounds for CanonicalQuickSort<P, SS>
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

impl<P, SS> HasSpace for CanonicalQuickSort<P, SS>
where
    P: PartitionScheme + HasSpace,
    SS: SmallSort + HasSpace,
{
    const SPACE: Complexity = Complexity::sum(
        Complexity::LOG_N,
        Complexity::sum(P::SPACE, SS::SPACE),
    );
}

impl<P, SS> HasStability for CanonicalQuickSort<P, SS>
where
    P: PartitionScheme + HasStability,
    SS: SmallSort + HasStability,
{
    const STABLE: bool = P::STABLE && SS::STABLE;
}
