use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(sort, "merge sort bottom up", "O(N Log(N))", true);

use super::utils::merge_inplace;

const SMALL_SORT_SIZE: usize = 32;
fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in (0..arr.len()).step_by(SMALL_SORT_SIZE) {
        let end = std::cmp::min(i + SMALL_SORT_SIZE, arr.len());
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(&mut arr[i..end], logger);
    }
    let mut tmp = logger.copy_aux_arr_t(arr);
    let mut gap = SMALL_SORT_SIZE;
    let mut i;
    while gap < arr.len() {
        i = 0;
        while i < arr.len() {
            let end = std::cmp::min(i + 2 * gap, arr.len());
            let mid = std::cmp::min(i + gap, arr.len());
            merge_inplace(&arr[i..mid], &arr[mid..end], &mut tmp[i..end], logger);
            i += gap * 2;
        }

        i = 0;
        gap *= 2;
        while i < arr.len() {
            let end = std::cmp::min(i + 2 * gap, arr.len());
            let mid = std::cmp::min(i + gap, arr.len());
            merge_inplace(&tmp[i..mid], &tmp[mid..end], &mut arr[i..end], logger);
            i += gap * 2;
        }
        gap *= 2;
    }
}
