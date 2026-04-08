use crate::traits::log_traits::SortLogger;
use super::{Rotation, forward_block_swap, buf_rotate_left, buf_rotate_right};

/// Bridge rotation (2021): minimize aux memory to bridge = |left − right|.
pub struct BridgeRotation;

impl Rotation for BridgeRotation {
    const NAME: &'static str = "bridge";
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
        if left < right {
            let bridge = right - left;
            if bridge < left {
                // Save bridge-sized gap, walk pairs backward, restore gap at front.
                let mut buf = logger.create_aux_arr_t(bridge);
                logger.copy_range(arr, left, &mut buf, 0, bridge);
                let mut ptb = left;
                let mut ptc = right;
                let mut ptd = n;
                for _ in 0..left {
                    ptc -= 1;
                    ptd -= 1;
                    ptb -= 1;
                    let v_ptd = arr[ptd];
                    logger.write_data(arr, ptc, v_ptd);
                    let v_ptb = arr[ptb];
                    logger.write_data(arr, ptd, v_ptb);
                }
                logger.copy_range(&buf, 0, arr, 0, bridge);
                logger.free_aux_arr_t(&buf);
            } else {
                buf_rotate_left(arr, left, logger);
            }
        } else {
            // right < left
            let bridge = left - right;
            if bridge < right {
                let mut buf = logger.create_aux_arr_t(bridge);
                logger.copy_range(arr, right, &mut buf, 0, bridge);
                let mut pta = 0usize;
                let mut ptb = left;
                let mut ptc = right;
                for _ in 0..right {
                    let v_pta = arr[pta];
                    logger.write_data(arr, ptc, v_pta);
                    let v_ptb = arr[ptb];
                    logger.write_data(arr, pta, v_ptb);
                    pta += 1;
                    ptb += 1;
                    ptc += 1;
                }
                logger.copy_range(&buf, 0, arr, n - bridge, bridge);
                logger.free_aux_arr_t(&buf);
            } else {
                buf_rotate_right(arr, left, logger);
            }
        }
    }
}

register_rotation!(BridgeRotation);
