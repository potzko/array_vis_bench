use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(sort, "rotate merge sort optimized", "O(N Log(N)^2)", true);

use super::utils::rotate_merge;

const SMALL_SORT_SIZE: usize = 32;
fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < SMALL_SORT_SIZE {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }

    let (left, right) = arr.split_at_mut(arr.len() / 2);
    sort(left, logger);
    sort(right, logger);

    if !logger.cmp_le(arr, arr.len() / 2 - 1, arr.len() / 2) {
        rotate_merge(arr, arr.len() / 2, logger)
    }
}
