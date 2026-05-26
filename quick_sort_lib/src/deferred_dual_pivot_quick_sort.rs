use std::marker::PhantomData;

use array_vis_bench_traits::DeferredSmallSort;
use sort_logger::SortLogger;

use array_vis_bench_traits::DualPivotSelector;

pub struct DeferredDualPivotQuickSort<DPS: DualPivotSelector, DSS: DeferredSmallSort>(
    PhantomData<(DPS, DSS)>,
);

impl<DPS: DualPivotSelector, DSS: DeferredSmallSort> DeferredDualPivotQuickSort<DPS, DSS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        deferred_dual_pivot_recursive::<T, U, DPS, DSS>(arr, logger);
        DSS::final_pass(arr, logger);
    }
}

fn deferred_dual_pivot_recursive<T, U, DPS, DSS>(arr: &mut [T], logger: &mut U)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    DPS: DualPivotSelector,
    DSS: DeferredSmallSort,
{
    if arr.len() < 2 {
        return;
    }
    if arr.len() <= DSS::THRESHOLD {
        return;
    }
    if arr.len() == 2 {
        logger.cond_swap_gt(arr, 0, 1);
        return;
    }

    let last = arr.len() - 1;

    let (p1_idx, p2_idx) = DPS::select(arr, logger);

    let p2_idx = {
        logger.swap(arr, p1_idx, 0);
        if p2_idx == p1_idx { 0 }
        else if p2_idx == 0  { p1_idx }
        else                  { p2_idx }
    };

    logger.swap(arr, p2_idx, last);

    if logger.cmp_gt(arr, 0, last) {
        logger.swap(arr, 0, last);
    }

    let p1 = arr[0];
    let p2 = arr[last];

    let mut lt = 1;
    let mut i  = 1;
    let mut gt = last - 1;

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

    deferred_dual_pivot_recursive::<T, U, DPS, DSS>(&mut arr[..lt], logger);
    if lt + 1 < gt {
        deferred_dual_pivot_recursive::<T, U, DPS, DSS>(&mut arr[lt + 1..gt], logger);
    }
    if gt + 1 < arr.len() {
        deferred_dual_pivot_recursive::<T, U, DPS, DSS>(&mut arr[gt + 1..], logger);
    }
}

// Same complexity profile as DualPivotQuickSort.
impl<DPS, DSS> array_vis_bench_traits::composable::HasTimeBounds for DeferredDualPivotQuickSort<DPS, DSS>
where
    DPS: array_vis_bench_traits::DualPivotSelector,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_SQUARED;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_LOG_N;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_LOG_N;
}
impl<DPS, DSS> array_vis_bench_traits::composable::HasSpace for DeferredDualPivotQuickSort<DPS, DSS>
where
    DPS: array_vis_bench_traits::DualPivotSelector,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const SPACE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::LOG_N;
}
impl<DPS, DSS> array_vis_bench_traits::composable::HasStability for DeferredDualPivotQuickSort<DPS, DSS>
where
    DPS: array_vis_bench_traits::DualPivotSelector,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const STABLE: bool = false;
}
