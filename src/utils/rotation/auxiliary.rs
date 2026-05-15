use crate::traits::log_traits::SortLogger;
use super::{Rotation, buf_rotate_left, buf_rotate_right};

/// Auxiliary rotation: copy the smaller side to a heap buffer (2021).
pub struct AuxiliaryRotation;

impl Rotation for AuxiliaryRotation {
    const NAME: &'static str = "auxiliary";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        _scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        let left = split_ind;
        let right = n - left;
        if left == 0 || right == 0 {
            return;
        }
        if left <= right {
            buf_rotate_left(arr, left, logger);
        } else {
            buf_rotate_right(arr, left, logger);
        }
    }
}

register_rotation!(AuxiliaryRotation);
