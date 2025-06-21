use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(sort, "merge sort optimized", "O(N Log(N))", true);

use super::utils::merge_inplace;

const SMALL_SORT_SIZE: usize = 32;
fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < SMALL_SORT_SIZE {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }
    let mut tmp = logger.copy_aux_arr_t(arr);
    merge_sort(arr, &mut tmp, logger);
}

fn merge_sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    target: &mut [T],
    logger: &mut U,
) {
    if arr.len() < SMALL_SORT_SIZE {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }
    let (left, right) = arr.split_at_mut(arr.len() / 2);
    let (left_target, right_target) = target.split_at_mut(target.len() / 2);

    merge_sort(left_target, left, logger);
    merge_sort(right_target, right, logger);
    merge_inplace(left_target, right_target, arr, logger);
}
