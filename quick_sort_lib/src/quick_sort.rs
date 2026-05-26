use std::marker::PhantomData;

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds, PivotQuality};
use array_vis_bench_traits::SmallSort;
use sort_logger::SortLogger;

use array_vis_bench_traits::PartitionScheme;
use array_vis_bench_traits::PivotSelector;

pub struct QuickSort<P: PartitionScheme, V: PivotSelector, SS: SmallSort>(
    PhantomData<(P, V, SS)>,
);

impl<P: PartitionScheme, V: PivotSelector, SS: SmallSort> QuickSort<P, V, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        quick_sort_recursive::<T, U, P, V, SS>(arr, logger);
    }
}

fn quick_sort_recursive<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    V: PivotSelector,
    SS: SmallSort,
>(
    arr: &mut [T],
    logger: &mut U,
) {
    if SS::THRESHOLD > 0 && arr.len() <= SS::THRESHOLD {
        SS::sort(arr, logger);
        return;
    }
    if arr.len() < 2 {
        return;
    }
    let pivot_idx = V::select(arr, logger);
    let (left_end, right_start) = P::partition(arr, logger, pivot_idx);
    quick_sort_recursive::<T, U, P, V, SS>(&mut arr[..left_end], logger);
    quick_sort_recursive::<T, U, P, V, SS>(&mut arr[right_start..], logger);
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
    V: PivotSelector + HasTimeBounds + PivotQuality,
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
    V: PivotSelector + HasSpace,
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
    V: PivotSelector + HasStability,
    SS: SmallSort + HasStability,
{
    const STABLE: bool = P::STABLE && V::STABLE && SS::STABLE;
}

