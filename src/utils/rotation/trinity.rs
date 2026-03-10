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
        if left < right {
            if left <= TRINITY_AUX {
                buf_rotate_left(arr, left, logger);
                return;
            }
            let bridge = right - left;
            if bridge <= TRINITY_AUX && bridge > 3 {
                let mut buf = logger.create_aux_arr_t(bridge);
                for i in 0..bridge {
                    logger.write_accross(arr, left + i, &mut buf, i);
                }
                let mut ptb = left;
                let mut ptc = right;
                let mut ptd = n;
                for _ in 0..left {
                    ptc -= 1; ptd -= 1; ptb -= 1;
                    let v_ptd = arr[ptd];
                    logger.write_data(arr, ptc, v_ptd);
                    let v_ptb = arr[ptb];
                    logger.write_data(arr, ptd, v_ptb);
                }
                for i in 0..bridge {
                    logger.write_data(arr, i, buf[i]);
                }
                logger.free_aux_arr_t(&buf);
            } else {
                ContrevRotation::rotate(arr, left, logger);
            }
        } else {
            // right < left
            if right <= TRINITY_AUX {
                buf_rotate_right(arr, left, logger);
                return;
            }
            let bridge = left - right;
            if bridge <= TRINITY_AUX && bridge > 3 {
                let mut buf = logger.create_aux_arr_t(bridge);
                for i in 0..bridge {
                    logger.write_accross(arr, right + i, &mut buf, i);
                }
                let mut pta = 0usize;
                let mut ptb = left;
                let mut ptc = right;
                for _ in 0..right {
                    let v_pta = arr[pta];
                    logger.write_data(arr, ptc, v_pta);
                    let v_ptb = arr[ptb];
                    logger.write_data(arr, pta, v_ptb);
                    pta += 1; ptb += 1; ptc += 1;
                }
                for i in 0..bridge {
                    logger.write_data(arr, n - bridge + i, buf[i]);
                }
                logger.free_aux_arr_t(&buf);
            } else {
                ContrevRotation::rotate(arr, left, logger);
            }
        }
    }
}
