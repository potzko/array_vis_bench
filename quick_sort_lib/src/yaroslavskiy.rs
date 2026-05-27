//! Dual-pivot partition — *Yaroslavskiy*'s scheme (Vladimir Yaroslavskiy),
//! as used in the JDK's `DualPivotQuicksort`. A [`PartitionScheme`] with
//! `N_PIVOTS = 2`.
//!
//! Takes two pivot indices, places them at the ends, scans inward
//! splitting `arr` into `< p1 | p1 ≤ x ≤ p2 | > p2`, and emits the
//! three unsorted regions through the visitor. The unified
//! `QuickSort<DualPivotPartition, V, SS>` (with `V: PivotInput<N = 2>`)
//! supersedes the old separate `DualPivotQuickSort` family.

use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionVisitor,
};
use sort_logger::SortLogger;

/// Dual-pivot partition — *Yaroslavskiy*'s scheme (Vladimir Yaroslavskiy).
pub struct DualPivotPartition;

impl PartitionScheme for DualPivotPartition {
    const NAME: &'static str = "dual pivot";
    const N_PIVOTS: usize = 2;

    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let len = arr.len();
        if len < 2 {
            return;
        }
        if len == 2 {
            logger.cond_swap_gt(arr, 0, 1);
            return;
        }
        let last = len - 1;
        let (p1_idx, p2_idx_raw) = (pivots[0], pivots[1]);

        // Place pivots at the ends.
        let p2_idx = {
            logger.swap(arr, p1_idx, 0);
            if p2_idx_raw == p1_idx { 0 }
            else if p2_idx_raw == 0 { p1_idx }
            else { p2_idx_raw }
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

        // Three unsorted regions; pivots at lt and gt are placed.
        visitor.unsorted(0..lt);
        if lt + 1 < gt {
            visitor.unsorted(lt + 1..gt);
        }
        if gt + 1 < len {
            visitor.unsorted(gt + 1..len);
        }
    }
}

// One pass over the array, no aux storage, not stable.
impl HasTimeBounds for DualPivotPartition {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for DualPivotPartition {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for DualPivotPartition {
    const STABLE: bool = false;
}
