const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "merge sort";

use super::utils::merge_inplace;
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
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut tmp = logger.copy_aux_arr_t(arr);
    merge_sort(arr, &mut tmp, logger);
}

const SMALL_SORT_SIZE: usize = 256;
use traits::sort_traits::SortAlgo;
fn merge_sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    target: &mut [T],
    logger: &mut U,
) {
    if arr.len() < SMALL_SORT_SIZE {
        super::super::shell_sorts::classic_shell_sorts::shell_optimized_256_elements::ShellSort::sort(arr, logger);
        return;
    }
    let (left, right) = arr.split_at_mut(arr.len() / 2);
    let (left_target, right_target) = target.split_at_mut(target.len() / 2);

    merge_sort(left_target, left, logger);
    merge_sort(right_target, right, logger);
    merge_inplace(left_target, right_target, arr, logger);
}
