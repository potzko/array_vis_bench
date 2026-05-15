use crate::traits::log_traits::SortLogger;
use super::{Rotation, unit_rotate_left, unit_rotate_right};

/// Helix rotation (2021): grail-derived, alternating inner loops.
pub struct HelixRotation;

impl Rotation for HelixRotation {
    const NAME: &'static str = "helix";
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
        loop {
            if left > right {
                if right <= 1 {
                    break;
                }
                while mid > start {
                    mid -= 1;
                    end -= 1;
                    logger.swap(arr, mid, end);
                }
                left %= right;
                mid += left;
                right = end - mid;
            } else {
                if left <= 1 {
                    break;
                }
                while mid < end {
                    logger.swap(arr, mid, start);
                    mid += 1;
                    start += 1;
                }
                right %= left;
                mid -= right;
                left = mid - start;
            }
        }
        // Loop exits with `min(left, right) <= 1`. If both > 0, one is 1 —
        // a unit rotation, fully in-place.
        if left == 1 && right > 0 {
            unit_rotate_left(&mut arr[start..end], logger);
        } else if right == 1 && left > 0 {
            unit_rotate_right(&mut arr[start..end], logger);
        }
    }
}

register_rotation!(HelixRotation);
