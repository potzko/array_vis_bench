use crate::traits::log_traits::SortLogger;
use super::{Rotation, forward_block_swap};

/// Piston rotation (2021): successive block swaps.
pub struct PistonRotation;

impl Rotation for PistonRotation {
    const NAME: &'static str = "piston";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
        let n = arr.len();
        let mut left = split_ind;
        let mut right = n - left;
        let mut start = 0usize;
        loop {
            if left == 0 {
                break;
            }
            while left <= right {
                forward_block_swap(arr, start, start + right, left, logger);
                right -= left;
            }
            if right == 0 {
                break;
            }
            loop {
                forward_block_swap(arr, start, start + left, right, logger);
                left -= right;
                start += right;
                if right > left {
                    break;
                }
            }
        }
    }
}

register_rotation!(PistonRotation);
