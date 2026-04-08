use crate::traits::log_traits::SortLogger;
use super::{Rotation, forward_block_swap, buf_rotate_left, buf_rotate_right};
use super::contrev::ContrevRotation;

const TRINITY_AUX: usize = 8;

/// Trinity rotation (2021): contrev + bridge, uses up to 8-element aux.
pub struct TrinityRotation;

impl Rotation for TrinityRotation {
    const NAME: &'static str = "trinity";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
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

        // Small side fits in aux: simple buffered rotation
        if small <= TRINITY_AUX {
            if left < right { buf_rotate_left(arr, left, logger) }
            else             { buf_rotate_right(arr, left, logger) }
            return;
        }

        // Bridge fits in aux: bridge rotation
        if bridge <= TRINITY_AUX && bridge > 3 {
            let mut buf = logger.create_aux_arr_t(bridge);

            if left < right {
                // Save bridge arr[left..right] to aux
                logger.copy_range(arr, left, &mut buf, 0, bridge);
                // Shift left part → tail, right tail → middle (backwards)
                let (mut ptb, mut ptc, mut ptd) = (left, right, n);
                for _ in 0..left {
                    ptc -= 1; ptd -= 1; ptb -= 1;
                    logger.write(arr, ptc, ptd);
                    logger.write(arr, ptd, ptb);
                }
                // Restore bridge from aux → front
                logger.copy_range(&buf, 0, arr, 0, bridge);
            } else {
                // Save bridge arr[right..left] to aux
                logger.copy_range(arr, right, &mut buf, 0, bridge);
                // Shift right part → front, left head → middle (forwards)
                let (mut pta, mut ptb, mut ptc) = (0, left, right);
                for _ in 0..right {
                    logger.write(arr, ptc, pta);
                    logger.write(arr, pta, ptb);
                    pta += 1; ptb += 1; ptc += 1;
                }
                // Restore bridge from aux → tail
                logger.copy_range(&buf, 0, arr, n - bridge, bridge);
            }

            logger.free_aux_arr_t(&buf);
            return;
        }

        // Fallback: contrev
        ContrevRotation::rotate(arr, left, logger);
    }
}

register_rotation!(TrinityRotation);
