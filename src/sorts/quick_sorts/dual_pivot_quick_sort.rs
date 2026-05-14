use std::marker::PhantomData;

use crate::utils::small_sort::SmallSort;
use crate::traits::log_traits::SortLogger;

use super::pivot_selectors::DualPivotSelector;

combo_codegen::family!(
    type = DualPivotQuickSort<{DPS}, {SS}>,
    uses = [
        "crate::utils::small_sort::{InsertionSmallSort, Network16SmallSort, NetworkSmallSort, NoSmallSort, Size1SmallSort, Size2SmallSort}",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "super::pivot_selectors::{FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther, CombinedSelector, NintherDualPivot}",
        "super::dual_pivot_quick_sort::DualPivotQuickSort",
    ],
    DPS: cross(PivotSelector, PivotSelector, "CombinedSelector<{0}, {1}>", "{0} / {1}")
       + [("NintherDualPivot", "ninther 1/3 + 2/3")],
    SS: SmallSort,
    name = "quick sort dual pivot",
    big_o = "O(N Log(N))",
    stable = false,
    direct_sort = true,
    path = ["quick sorts", "dual pivot", "{DPS}", "{SS}"],
);

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
