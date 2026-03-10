use crate::traits::log_traits::SortLogger;
use super::{Rotation, forward_block_swap, backward_block_swap, buf_rotate_left};

/// Grail rotation (2020): Gries-Mills with a stack-based aux at the end.
pub struct GrailRotation;

impl Rotation for GrailRotation {
    const NAME: &'static str = "grail";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
        let n = arr.len();
        let mut left = split_ind;
        let mut right = n - left;
        let mut start = 0usize;
        let mut min = left.min(right);
        while min > 1 {
            if left <= right {
                loop {
                    forward_block_swap(arr, start, start + left, left, logger);
                    start += left;
                    right -= left;
                    if left > right {
                        break;
                    }
                }
                min = right;
            } else {
                loop {
                    backward_block_swap(arr, start + left - right, start + left, right, logger);
                    left -= right;
                    if right > left {
                        break;
                    }
                }
                min = left;
            }
        }
        if left > 0 && right > 0 {
            buf_rotate_left(&mut arr[start..start + left + right], left, logger);
        }
    }
}
