use std::marker::PhantomData;

use crate::sorts::merge_sorts::small_sort::SmallSort;
use crate::traits::log_traits::SortLogger;

use super::pivot_selectors::PivotSelector;

/// Dual-pivot quicksort (Yaroslavskiy partition).
///
/// Selects two pivots using independent strategies: `V1` picks the first pivot
/// from the full array and swaps it to index 0, then `V2` picks the second
/// pivot from `arr[1..]`. The two pivots partition into three regions
/// `< p1`, `p1 ≤ x ≤ p2`, `> p2`, then recurses on all three.
pub struct DualPivotQuickSort<V1: PivotSelector, V2: PivotSelector, SS: SmallSort>(
    PhantomData<(V1, V2, SS)>,
);

impl<V1: PivotSelector, V2: PivotSelector, SS: SmallSort> DualPivotQuickSort<V1, V2, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        dual_pivot_recursive::<T, U, V1, V2, SS>(arr, logger);
    }
}

fn dual_pivot_recursive<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    V1: PivotSelector,
    V2: PivotSelector,
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
    if arr.len() == 2 {
        logger.cond_swap_gt(arr, 0, 1);
        return;
    }

    // V1 selects first pivot from the full array, swap to index 0
    let p1_idx = V1::select(arr, logger);
    logger.swap(arr, p1_idx, 0);

    // V2 selects second pivot from arr[1..], swap to last index
    let last = arr.len() - 1;
    let p2_idx = 1 + V2::select(&arr[1..], logger);
    logger.swap(arr, p2_idx, last);

    // Ensure arr[0] <= arr[last] (p1 <= p2)
    if logger.cmp_gt(arr, 0, last) {
        logger.swap(arr, 0, last);
    }

    let p1 = arr[0];
    let p2 = arr[last];

    let mut lt = 1; // boundary of "< p1" (exclusive)
    let mut i = 1; // scan pointer
    let mut gt = last - 1; // boundary of "> p2" (inclusive)

    while i <= gt {
        if logger.cmp_lt_data(arr, i, p1) {
            logger.swap(arr, i, lt);
            lt += 1;
            i += 1;
        } else if logger.cmp_gt_data(arr, i, p2) {
            // Skip elements at gt that are already > p2
            while i < gt && logger.cmp_gt_data(arr, gt, p2) {
                gt -= 1;
            }
            logger.swap(arr, i, gt);
            if gt == 0 {
                break;
            }
            gt -= 1;
            // Re-examine the swapped-in element
            if logger.cmp_lt_data(arr, i, p1) {
                logger.swap(arr, i, lt);
                lt += 1;
            }
            i += 1;
        } else {
            // p1 <= arr[i] <= p2
            i += 1;
        }
    }

    // Place pivots in their final positions
    lt -= 1;
    gt += 1;
    logger.swap(arr, 0, lt);
    logger.swap(arr, last, gt);

    // Recurse on three regions (pivots at lt and gt are already placed)
    dual_pivot_recursive::<T, U, V1, V2, SS>(&mut arr[..lt], logger);
    if lt + 1 < gt {
        dual_pivot_recursive::<T, U, V1, V2, SS>(&mut arr[lt + 1..gt], logger);
    }
    if gt + 1 < arr.len() {
        dual_pivot_recursive::<T, U, V1, V2, SS>(&mut arr[gt + 1..], logger);
    }
}
