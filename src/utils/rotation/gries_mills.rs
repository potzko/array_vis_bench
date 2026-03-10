use crate::traits::log_traits::SortLogger;
use super::{Rotation, forward_block_swap, backward_block_swap};

/// Gries-Mills rotation (1981).
pub struct GriesMillsRotation;

impl Rotation for GriesMillsRotation {
    const NAME: &'static str = "gries-mills";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
        let n = arr.len();
        let mut left = split_ind;
        let mut right = n - left;
        let mut start = 0usize;
        while left != 0 && right != 0 {
            if left <= right {
                loop {
                    forward_block_swap(arr, start, start + left, left, logger);
                    start += left;
                    right -= left;
                    if left > right {
                        break;
                    }
                }
            } else {
                loop {
                    backward_block_swap(arr, start + left - right, start + left, right, logger);
                    left -= right;
                    if right > left {
                        break;
                    }
                }
            }
        }
    }
}
