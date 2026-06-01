use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionVisitor,
};
use sort_logger::SortLogger;

/// Block partition (batched compare-then-swap).
///
/// Processes elements in fixed-size blocks: first classifies which
/// elements need to move (into offset buffers), then swaps them in a
/// tight loop. Reduces branch mispredictions compared to LeftLeftPartition/LeftRightPartition.
pub struct Block;

/// Per-side offset-buffer length. The scheme needs two of these (left
/// and right), so [`Block::SCRATCH_LEN`] is `2 * BLOCK`.
const BLOCK: usize = 64;

impl PartitionScheme for Block {
    const NAME: &'static str = "block";
    const N_PIVOTS: usize = 1;
    /// Two `BLOCK`-sized offset buffers, allocated once by the driver and
    /// reused for the whole sort instead of re-created per partition call.
    const SCRATCH_LEN: usize = 2 * BLOCK;
    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        scratch: &mut [usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let len = arr.len();
        logger.swap(arr, pivots[0], len - 1);
        let pivot = arr[len - 1];

        // `scratch` is the driver's reusable buffer (length
        // `SCRATCH_LEN == 2 * BLOCK`), logged once for the whole sort.
        // The two offset regions are the lower/upper halves; we index
        // them with absolute positions so every logged write targets the
        // single persistent aux array rather than re-announcing a buffer
        // on each call.
        debug_assert!(scratch.len() >= 2 * BLOCK);
        let off_r = BLOCK;

        let mut left = 0;
        let mut right = len - 1;

        while right - left > 2 * BLOCK {
            let mut num_l = 0;
            for i in 0..BLOCK {
                if logger.cmp_gt_data(arr, left + i, pivot) {
                    logger.write_data_u(scratch, num_l, i);
                    num_l += 1;
                }
            }
            let mut num_r = 0;
            for i in 0..BLOCK {
                if logger.cmp_le_data(arr, right - 1 - i, pivot) {
                    logger.write_data_u(scratch, off_r + num_r, i);
                    num_r += 1;
                }
            }
            let swaps = num_l.min(num_r);
            for s in 0..swaps {
                logger.swap(arr, left + scratch[s], right - 1 - scratch[off_r + s]);
            }
            if num_l <= num_r {
                left += BLOCK;
            }
            if num_r <= num_l {
                right -= BLOCK;
            }
        }

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
