use sort_logger::SortLogger;
use array_vis_bench_traits::role::rotation::{
    buf_rotate_left_using, buf_rotate_right_using, forward_block_swap, Rotation,
};
use rotation_contrev::ContrevRotation;

const TRINITY_AUX: usize = 8;

/// Trinity rotation (2021): contrev + bridge, uses up to 8-element aux.
///
/// In the original C, that aux is a stack-resident `T tmp[8]`. Here we
/// expose it through the trait's [`scratch_size`](Rotation::scratch_size)
/// so the caller (a rotation merge sort or the standalone runner) can
/// pre-allocate and *register* the 8-element scratch buffer once for the
/// whole sort run — the visualiser then shows a single aux array rather
/// than a fresh one per rotation call.
pub struct TrinityRotation;

impl Rotation for TrinityRotation {
    const NAME: &'static str = "trinity";

    #[inline]
    fn scratch_size(_n: usize) -> usize {
        TRINITY_AUX
    }

    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        let left = split_ind;
        let right = n - left;

        if left == 0 || right == 0 {
            return;
        }
        if left == right {
            forward_block_swap(arr, 0, left, left, logger);
            return;
        }

        let small = left.min(right);
        let bridge = left.abs_diff(right);

        // Small side fits in aux: buffered rotation using the registered
        // scratch (slot 0..small).
        if small <= TRINITY_AUX {
            if left < right {
                buf_rotate_left_using(arr, left, scratch, logger);
            } else {
                buf_rotate_right_using(arr, left, scratch, logger);
            }
            return;
        }

        // Bridge fits in aux: bridge rotation, also using the registered
        // scratch (slot 0..bridge).
        if bridge <= TRINITY_AUX && bridge > 3 {
            if left < right {
                // Save bridge arr[left..right] to scratch
                logger.copy_range(arr, left, scratch, 0, bridge);
                // Shift left part → tail, right tail → middle (backwards)
                let (mut ptb, mut ptc, mut ptd) = (left, right, n);
                for _ in 0..left {
                    ptc -= 1; ptd -= 1; ptb -= 1;
                    logger.write(arr, ptc, ptd);
                    logger.write(arr, ptd, ptb);
                }
                // Restore bridge from scratch → front
                logger.copy_range(scratch, 0, arr, 0, bridge);
            } else {
                // Save bridge arr[right..left] to scratch
                logger.copy_range(arr, right, scratch, 0, bridge);
                // Shift right part → front, left head → middle (forwards)
                let (mut pta, mut ptb, mut ptc) = (0, left, right);
                for _ in 0..right {
                    logger.write(arr, ptc, pta);
                    logger.write(arr, pta, ptb);
                    pta += 1; ptb += 1; ptc += 1;
                }
                // Restore bridge from scratch → tail
                logger.copy_range(scratch, 0, arr, n - bridge, bridge);
            }
            return;
        }

        // Fallback: contrev (in-place, doesn't need scratch).
        ContrevRotation::rotate(arr, left, &mut [], logger);
    }
}

impl array_vis_bench_traits::HasSpace for TrinityRotation {
    /// Fixed `TRINITY_AUX` (8-element) scratch — independent of N.
    const SPACE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::CONST;
}


// Uniform across all rotations: O(N), not stable. Wired into the
// standalone-algorithm registry by `register_rotation!` in array_vis_bench;
// kept here because the orphan rule requires the trait impl to live with
// the type.
impl array_vis_bench_traits::HasTimeBounds for TrinityRotation {
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N1;
}
impl array_vis_bench_traits::HasStability for TrinityRotation {
    const STABLE: bool = false;
}
