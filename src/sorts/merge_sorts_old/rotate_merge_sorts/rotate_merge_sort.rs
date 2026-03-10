use crate::create_sort;

create_sort!(sort, "rotate merge sort", "O(N Log(N)^2)", true);

use super::utils::rotate_merge;

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    let (left, right) = arr.split_at_mut(arr.len() / 2);
    sort(left, logger);
    sort(right, logger);
    rotate_merge(arr, arr.len() / 2, logger);
}
