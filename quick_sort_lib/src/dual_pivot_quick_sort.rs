use std::marker::PhantomData;

use array_vis_bench_traits::SmallSort;
use sort_logger::SortLogger;

use array_vis_bench_traits::DualPivotSelector;

/// Dual-pivot quicksort (Yaroslavskiy partition).
///
/// `DPS` returns two pivot indices; the algorithm swaps them to the ends,
/// partitions into `< p1 | p1 ≤ x ≤ p2 | > p2`, then recurses on all three
/// regions.
pub struct DualPivotQuickSort<DPS: DualPivotSelector, SS: SmallSort>(
    PhantomData<(DPS, SS)>,
);

impl<DPS: DualPivotSelector, SS: SmallSort> DualPivotQuickSort<DPS, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        dual_pivot_recursive::<T, U, DPS, SS>(arr, logger);
    }
}

fn dual_pivot_recursive<T, U, DPS, SS>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    DPS: DualPivotSelector,
    SS: SmallSort,
{
    if SS::THRESHOLD > 0 && arr.len() <= SS::THRESHOLD {
        SS::sort(arr, logger);
        return;
    }
    if arr.len() < 2 {
        return;
    }
    if arr.len() == 2 {
        logger.cond_swap_gt(arr, 0, 1);
        return;
    }

    let last = arr.len() - 1;

    // Ask DPS for two raw indices, then place them at the ends.
    let (p1_idx, p2_idx) = DPS::select(arr, logger);

    // Swap p1 to index 0, keeping p2_idx valid across the swap.
    let p2_idx = {
        logger.swap(arr, p1_idx, 0);
        // The swap exchanged positions p1_idx ↔ 0; update p2_idx accordingly.
        if p2_idx == p1_idx { 0 }
        else if p2_idx == 0  { p1_idx }
        else                  { p2_idx }
    };

    // Swap p2 to last index.
    logger.swap(arr, p2_idx, last);

    // Normalise so arr[0] (p1) ≤ arr[last] (p2).
    if logger.cmp_gt(arr, 0, last) {
        logger.swap(arr, 0, last);
    }

    let p1 = arr[0];
    let p2 = arr[last];

    let mut lt = 1;        // exclusive end of "< p1" region
    let mut i  = 1;        // scan pointer
    let mut gt = last - 1; // inclusive start of "> p2" region

    while i <= gt {
        if logger.cmp_lt_data(arr, i, p1) {
            logger.swap(arr, i, lt);
            lt += 1;
            i += 1;
        } else if logger.cmp_gt_data(arr, i, p2) {
            while i < gt && logger.cmp_gt_data(arr, gt, p2) {
                gt -= 1;
            }
            logger.swap(arr, i, gt);
            if gt == 0 {
                break;
            }
            gt -= 1;
            if logger.cmp_lt_data(arr, i, p1) {
                logger.swap(arr, i, lt);
                lt += 1;
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    lt -= 1;
    gt += 1;
    logger.swap(arr, 0, lt);
    logger.swap(arr, last, gt);

    dual_pivot_recursive::<T, U, DPS, SS>(&mut arr[..lt], logger);
    if lt + 1 < gt {
        dual_pivot_recursive::<T, U, DPS, SS>(&mut arr[lt + 1..gt], logger);
    }
    if gt + 1 < arr.len() {
        dual_pivot_recursive::<T, U, DPS, SS>(&mut arr[gt + 1..], logger);
    }
}

// Dual-pivot quicksort: complexity is dominated by the partition phase
// (O(N) per level) over log-N depth. With well-chosen dual pivots, all
// three partitions stay balanced; with degenerate pivots, depth can
// blow up to O(N) — same worst-case profile as single-pivot QuickSort.
// We don't currently model DualPivotSelector quality, so we declare a
// conservative O(N²) worst and O(N log N) best/average.
impl<DPS, SS> array_vis_bench_traits::composable::HasTimeBounds for DualPivotQuickSort<DPS, SS>
where
    DPS: array_vis_bench_traits::DualPivotSelector,
    SS: array_vis_bench_traits::SmallSort,
{
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_SQUARED;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_LOG_N;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_LOG_N;
}
impl<DPS, SS> array_vis_bench_traits::composable::HasSpace for DualPivotQuickSort<DPS, SS>
where
    DPS: array_vis_bench_traits::DualPivotSelector,
    SS: array_vis_bench_traits::SmallSort,
{
    const SPACE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::LOG_N;
}
impl<DPS, SS> array_vis_bench_traits::composable::HasStability for DualPivotQuickSort<DPS, SS>
where
    DPS: array_vis_bench_traits::DualPivotSelector,
    SS: array_vis_bench_traits::SmallSort,
{
    const STABLE: bool = false;
}
