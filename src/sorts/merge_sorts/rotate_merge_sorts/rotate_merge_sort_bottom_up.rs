use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(sort, "rotate merge sort bottom up", "O(N Log(N)^2)", true);

use super::utils::rotate_merge;

const SMALL_SORT_SIZE: usize = 20;
fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in (0..arr.len()).step_by(SMALL_SORT_SIZE) {
        let end = std::cmp::min(i + SMALL_SORT_SIZE, arr.len());
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(&mut arr[i..end], logger);
    }
    let mut gap = SMALL_SORT_SIZE;
    while gap < arr.len() {
        let mut i = 0;
        while i < arr.len() {
            let end = std::cmp::min(i + 2 * gap, arr.len());
            if i + gap < arr.len() {
                rotate_merge(&mut arr[i..end], gap, logger);
            }
            i += gap * 2;
        }
        gap *= 2;
    }
}
