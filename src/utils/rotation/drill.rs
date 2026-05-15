use crate::traits::log_traits::SortLogger;
use super::{Rotation, unit_rotate_left, unit_rotate_right};

/// Drill rotation (2021): grail + piston + helix inner loops combined.
pub struct DrillRotation;

impl Rotation for DrillRotation {
    const NAME: &'static str = "drill";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        split_ind: usize,
        _scratch: &mut [T],
        logger: &mut U,
    ) {
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
        // Outer-loop exit guarantees `min(left, right) <= 1`. With both > 0,
        // exactly one of them is 1 — a unit rotation, fully in-place.
        if left == 1 && right > 0 {
            unit_rotate_left(&mut arr[start..end], logger);
        } else if right == 1 && left > 0 {
            unit_rotate_right(&mut arr[start..end], logger);
        }
    }
}

register_rotation!(DrillRotation);
