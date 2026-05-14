use crate::traits::log_traits::SortLogger;

pub trait PartitionScheme {
    /// Display name used both in the `Partition` component slot and in
    /// the per-algorithm path the menu builds at startup.
    const NAME: &'static str;
    /// Partition `arr` with the pivot originally at `pivot_idx`.
    ///
    /// Returns `(left_end, right_start)`:
    /// - `arr[..left_end]` needs further sorting
    /// - `arr[right_start..]` needs further sorting
    /// - `arr[left_end..right_start]` is already placed
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize);
}

/// Lomuto partition (left-left single-pointer scan).
///
/// Moves the pivot to the end, scans left-to-right placing small elements
/// at the front, then swaps the pivot into its final position.
pub struct Lomuto;
combo_codegen::component!(Partition, Lomuto, "lomuto");

impl PartitionScheme for Lomuto {
    const NAME: &'static str = "lomuto";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        let len = arr.len();
        logger.swap(arr, pivot_idx, len - 1);
        let pivot = arr[len - 1];

        let mut small = 0;
        for i in 0..len - 1 {
            if logger.cmp_le_data(arr, i, pivot) {
                logger.swap(arr, i, small);
                small += 1;
            }
        }
        logger.swap(arr, small, len - 1);
        (small, small + 1)
    }
}

/// Hoare partition (left-right two-pointer scan).
///
/// Moves the pivot to the start, scans inward from both ends, then swaps the
/// pivot into its final position.
pub struct Hoare;
combo_codegen::component!(Partition, Hoare, "hoare");

impl PartitionScheme for Hoare {
    const NAME: &'static str = "hoare";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);
        let pivot = arr[0];

        let mut left = 1;
        let mut right = arr.len() - 1;
        while left <= right {
            while left <= right && logger.cmp_le_data(arr, left, pivot) {
                left += 1;
            }
            while left <= right && logger.cmp_gt_data(arr, right, pivot) {
                right -= 1;
            }
            if left < right {
                logger.swap(arr, left, right);
                left += 1;
                right -= 1;
            }
        }
        logger.swap(arr, 0, right);
        (right, right + 1)
    }
}

/// Three-way partition (Dutch National Flag).
///
/// Splits into three regions: `< pivot`, `== pivot`, `> pivot`.
/// Equal elements are grouped in the middle and excluded from recursion.
pub struct ThreeWay;
combo_codegen::component!(Partition, ThreeWay, "three-way");

impl PartitionScheme for ThreeWay {
    const NAME: &'static str = "three-way";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);
        let pivot = arr[0];

        let mut lt = 0; // end of "< pivot" region
        let mut i = 1; // scan pointer
        let mut gt = arr.len() - 1; // start of "> pivot" region

        while i <= gt {
            if logger.cmp_lt_data(arr, i, pivot) {
                logger.swap(arr, i, lt);
                lt += 1;
                i += 1;
            } else if logger.cmp_gt_data(arr, i, pivot) {
                logger.swap(arr, i, gt);
                if gt == 0 {
                    break;
                }
                gt -= 1;
                // don't advance i — swapped-in element not yet examined
            } else {
                i += 1; // == pivot
            }
        }
        (lt, gt + 1)
    }
}

/// Block partition (batched compare-then-swap).
///
/// Processes elements in fixed-size blocks: first classifies which elements
/// need to move (into offset buffers), then swaps them in a tight loop.
/// Reduces branch mispredictions compared to Lomuto/Hoare.
pub struct Block;
combo_codegen::component!(Partition, Block, "block");

impl PartitionScheme for Block {
    const NAME: &'static str = "block";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        let len = arr.len();
        logger.swap(arr, pivot_idx, len - 1);
        let pivot = arr[len - 1];

        const BLOCK: usize = 64;
        let mut offsets_l = [0usize; BLOCK];
        let mut offsets_r = [0usize; BLOCK];
        logger.log_aux_arr_u(&offsets_l);
        logger.log_aux_arr_u(&offsets_r);

        let mut left = 0;
        let mut right = len - 1; // pivot is at len-1

        while right - left > 2 * BLOCK {
            // Phase 1: classify left block
            let mut num_l = 0;
            for i in 0..BLOCK {
                if logger.cmp_gt_data(arr, left + i, pivot) {
                    logger.write_data_u(&mut offsets_l, num_l, i);
                    num_l += 1;
                }
            }
            // Phase 1: classify right block
            let mut num_r = 0;
            for i in 0..BLOCK {
                if logger.cmp_le_data(arr, right - 1 - i, pivot) {
                    logger.write_data_u(&mut offsets_r, num_r, i);
                    num_r += 1;
                }
            }
            // Phase 2: swap matching pairs
            let swaps = num_l.min(num_r);
            for s in 0..swaps {
                logger.swap(arr, left + offsets_l[s], right - 1 - offsets_r[s]);
            }
            // Advance the side that was fully consumed
            if num_l <= num_r {
                left += BLOCK;
            }
            if num_r <= num_l {
                right -= BLOCK;
            }
        }

        logger.free_aux_arr(&offsets_l);
        logger.free_aux_arr(&offsets_r);

        // Remainder: fall back to Lomuto for the leftover elements
        let mut small = left;
        for i in left..right {
            if logger.cmp_le_data(arr, i, pivot) {
                logger.swap(arr, i, small);
                small += 1;
            }
        }
        logger.swap(arr, small, len - 1);
        (small, small + 1)
    }
}

/// Moving-pivot partition.
///
/// Swaps the selected pivot to the start, then walks inward: elements smaller
/// than the current head extend the low region, larger elements are swapped to
/// the high end.
pub struct MovingPivot;
combo_codegen::component!(Partition, MovingPivot, "moving pivot");

impl PartitionScheme for MovingPivot {
    const NAME: &'static str = "moving pivot";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);

        let mut low = 0;
        let mut high = arr.len() - 1;
        while low < high - 1 {
            if logger.cond_swap_le(arr, low + 1, low) {
                low += 1;
            } else {
                logger.swap(arr, low + 1, high);
                high -= 1;
            }
        }
        logger.cond_swap_lt(arr, high, low);
        (high, high)
    }
}
