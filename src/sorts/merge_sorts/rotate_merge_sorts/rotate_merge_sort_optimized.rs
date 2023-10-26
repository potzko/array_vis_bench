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

const SMALL_SORT_SIZE: usize = 32;
use traits::sort_traits::SortAlgo;
fn rotate_merge_sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    if arr.len() < SMALL_SORT_SIZE {
        crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, logger);
        return;
    }

    {
        let (left, right) = arr.split_at_mut(arr.len() / 2);
        rotate_merge_sort(left, logger);
        rotate_merge_sort(right, logger);
    }
    if !logger.cmp_le(arr, arr.len() / 2 - 1, arr.len() / 2) {
        rotate_merge(arr, arr.len() / 2, logger)
    }
}
