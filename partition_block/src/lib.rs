use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionSchemeV,
    PartitionVisitor,
};
use sort_logger::SortLogger;

/// Block partition (batched compare-then-swap).
///
/// Processes elements in fixed-size blocks: first classifies which
/// elements need to move (into offset buffers), then swaps them in a
/// tight loop. Reduces branch mispredictions compared to Lomuto/Hoare.
pub struct Block;

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
            if num_l <= num_r {
                left += BLOCK;
            }
            if num_r <= num_l {
                right -= BLOCK;
            }
        }

        logger.free_aux_arr(&offsets_l);
        logger.free_aux_arr(&offsets_r);

        // Remainder: fall back to Lomuto-style for the leftover elements.
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

impl PartitionSchemeV for Block {
    const NAME: &'static str = "block";
    const N_PIVOTS: usize = 1;
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
        logger.swap(arr, pivots[0], len - 1);
        let pivot = arr[len - 1];

        const BLOCK: usize = 64;
        let mut offsets_l = [0usize; BLOCK];
        let mut offsets_r = [0usize; BLOCK];
        logger.log_aux_arr_u(&offsets_l);
        logger.log_aux_arr_u(&offsets_r);

        let mut left = 0;
        let mut right = len - 1;

        while right - left > 2 * BLOCK {
            let mut num_l = 0;
            for i in 0..BLOCK {
                if logger.cmp_gt_data(arr, left + i, pivot) {
                    logger.write_data_u(&mut offsets_l, num_l, i);
                    num_l += 1;
                }
            }
            let mut num_r = 0;
            for i in 0..BLOCK {
                if logger.cmp_le_data(arr, right - 1 - i, pivot) {
                    logger.write_data_u(&mut offsets_r, num_r, i);
                    num_r += 1;
                }
            }
            let swaps = num_l.min(num_r);
            for s in 0..swaps {
                logger.swap(arr, left + offsets_l[s], right - 1 - offsets_r[s]);
            }
            if num_l <= num_r {
                left += BLOCK;
            }
            if num_r <= num_l {
                right -= BLOCK;
            }
        }

        logger.free_aux_arr(&offsets_l);
        logger.free_aux_arr(&offsets_r);

        let mut small = left;
        for i in left..right {
            if logger.cmp_le_data(arr, i, pivot) {
                logger.swap(arr, i, small);
                small += 1;
            }
        }
        logger.swap(arr, small, len - 1);
        visitor.unsorted(0..small);
        visitor.unsorted(small + 1..len);
    }
}

impl HasTimeBounds for Block {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for Block {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Block {
    const STABLE: bool = false;
}
