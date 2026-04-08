use crate::traits::log_traits::SortLogger;
use super::{Rotation, buf_rotate_left};

/// Drill rotation (2021): grail + piston + helix inner loops combined.
pub struct DrillRotation;

impl Rotation for DrillRotation {
    const NAME: &'static str = "drill";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
        let n = arr.len();
        let mut left = split_ind;
        let mut right = n - left;
        let mut start = 0usize;
        let mut end = n;
        let mut mid = left;
        while left > 1 {
            if left <= right {
                right %= left;
                let loop_count = end - mid - right;
                for _ in 0..loop_count {
                    logger.swap(arr, mid, start);
                    mid += 1;
                    start += 1;
                }
            }
            if right <= 1 {
                break;
            }
            left %= right;
            let loop_count = mid - start - left;
            for _ in 0..loop_count {
                mid -= 1;
                end -= 1;
                logger.swap(arr, mid, end);
            }
        }
        if left != 0 && right != 0 {
            buf_rotate_left(&mut arr[start..end], left, logger);
        }
    }
}

register_rotation!(DrillRotation);
