//! Quickselect built on Yaroslavskiy's dual-pivot partition.
//!
//! Same role as the single-pivot QuickSelect, but each step splits into
//! three regions — `< p1 | p1 ≤ x ≤ p2 | > p2` — and recurses into the
//! single region containing `target`. The other two sub-arrays are left
//! unordered.

use std::marker::PhantomData;

use array_vis_bench_traits::{
    Complexity, DualPivotSelector, HasSpace, HasStability, HasTimeBounds, QuickSelect,
};
use sort_logger::SortLogger;

// ── RecursiveDualPivotQuickSelect ────────────────────────────────────────────

pub struct RecursiveDualPivotQuickSelect<DPS: DualPivotSelector>(PhantomData<DPS>);

impl<DPS: DualPivotSelector> QuickSelect for RecursiveDualPivotQuickSelect<DPS> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    ) {
        recursive::<T, U, DPS>(arr, logger, target);
    }
}

fn recursive<T, U, DPS>(arr: &mut [T], logger: &mut U, target: usize)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    DPS: DualPivotSelector,
{
    if arr.len() < 2 {
        return;
    }
    if arr.len() == 2 {
        logger.cond_swap_gt(arr, 0, 1);
        return;
    }
    let (lt, gt) = dual_pivot_partition::<T, U, DPS>(arr, logger);
    if target < lt {
        recursive::<T, U, DPS>(&mut arr[..lt], logger, target);
    } else if target > lt && target < gt {
        recursive::<T, U, DPS>(&mut arr[lt + 1..gt], logger, target - (lt + 1));
    } else if target > gt {
        recursive::<T, U, DPS>(&mut arr[gt + 1..], logger, target - (gt + 1));
    }
    // target == lt or target == gt → pivot is already in its final spot.
}

// ── IterativeDualPivotQuickSelect ────────────────────────────────────────────

pub struct IterativeDualPivotQuickSelect<DPS: DualPivotSelector>(PhantomData<DPS>);

impl<DPS: DualPivotSelector> QuickSelect for IterativeDualPivotQuickSelect<DPS> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    ) {
        let mut lo = 0usize;
        let mut hi = arr.len();
        let mut target = target;
        loop {
            let len = hi - lo;
            if len < 2 {
                return;
            }
            if len == 2 {
                logger.cond_swap_gt(arr, lo, lo + 1);
                return;
            }
            let (lt, gt) = dual_pivot_partition::<T, U, DPS>(&mut arr[lo..hi], logger);
            if target < lt {
                hi = lo + lt;
            } else if target > lt && target < gt {
                let new_lo = lo + lt + 1;
                let new_hi = lo + gt;
                lo = new_lo;
                hi = new_hi;
                target -= lt + 1;
            } else if target > gt {
                let new_lo = lo + gt + 1;
                lo = new_lo;
                target -= gt + 1;
            } else {
                return;
            }
        }
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// `DualPivotSelector` quality isn't modelled with `PivotQuality` yet,
// so worst case is conservatively O(N²). Average / best are O(N) — one
// O(N) Yaroslavskiy partition per level, expected constant levels.
macro_rules! impl_dp_qs_annotations {
    ($ty:ident, $space:expr) => {
        impl<DPS: DualPivotSelector> HasTimeBounds for $ty<DPS> {
            const WORST: Complexity = Complexity::N_SQUARED;
            const BEST: Complexity = Complexity::N1;
            const AVERAGE: Complexity = Complexity::N1;
        }
        impl<DPS: DualPivotSelector> HasSpace for $ty<DPS> {
            const SPACE: Complexity = $space;
        }
        impl<DPS: DualPivotSelector> HasStability for $ty<DPS> {
            const STABLE: bool = false;
        }
    };
}

impl_dp_qs_annotations!(RecursiveDualPivotQuickSelect, Complexity::LOG_N);
impl_dp_qs_annotations!(IterativeDualPivotQuickSelect, Complexity::CONST);

// ── Yaroslavskiy dual-pivot partition ────────────────────────────────────────

/// Partition `arr` around two pivots chosen by `DPS`. Returns `(lt, gt)`
/// where after the call:
///
/// - `arr[..lt]`        — strictly less than the first pivot
/// - `arr[lt]`          — first pivot (in final position)
/// - `arr[lt + 1..gt]`  — between the two pivots (inclusive on both ends)
/// - `arr[gt]`          — second pivot (in final position)
/// - `arr[gt + 1..]`    — strictly greater than the second pivot
fn dual_pivot_partition<T, U, DPS>(arr: &mut [T], logger: &mut U) -> (usize, usize)
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    DPS: DualPivotSelector,
{
    let last = arr.len() - 1;

    let (p1_idx, p2_idx) = DPS::select(arr, logger);

    let p2_idx = {
        logger.swap(arr, p1_idx, 0);
        if p2_idx == p1_idx {
            0
        } else if p2_idx == 0 {
            p1_idx
        } else {
            p2_idx
        }
    };
    logger.swap(arr, p2_idx, last);

    if logger.cmp_gt(arr, 0, last) {
        logger.swap(arr, 0, last);
    }

    let p1 = arr[0];
    let p2 = arr[last];

    let mut lt = 1;
    let mut i = 1;
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

    (lt, gt)
}
