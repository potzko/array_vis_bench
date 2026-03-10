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
                    let tmp = arr[mid];
                    logger.write_data(arr, mid, arr[start]);
                    logger.write_data(arr, start, tmp);
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
                let tmp = arr[mid];
                logger.write_data(arr, mid, arr[end]);
                logger.write_data(arr, end, tmp);
            }
        }
        if left != 0 && right != 0 {
            buf_rotate_left(&mut arr[start..end], left, logger);
        }
    }
}
