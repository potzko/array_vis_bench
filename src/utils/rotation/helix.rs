use crate::traits::log_traits::SortLogger;
use super::{Rotation, buf_rotate_left};

/// Helix rotation (2021): grail-derived, alternating inner loops.
pub struct HelixRotation;

impl Rotation for HelixRotation {
    const NAME: &'static str = "helix";
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
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
        if left != 0 && right != 0 {
            buf_rotate_left(&mut arr[start..end], left, logger);
        }
    }
}

register_rotation!(HelixRotation);
