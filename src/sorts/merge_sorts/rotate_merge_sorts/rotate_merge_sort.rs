const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N)^2)";
const NAME: &str = "merge sort";

use super::utils::rotate_merge;
use crate::traits;
pub struct MergeSort {}

impl traits::sort_traits::SortAlgo for MergeSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        rotate_merge_sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn rotate_merge_sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    if arr.len() < 2 {
        return;
    }

    {
        let (left, right) = arr.split_at_mut(arr.len() / 2);
        rotate_merge_sort(left, logger);
        rotate_merge_sort(right, logger);
    }
    rotate_merge(arr, arr.len() / 2, logger)
}
