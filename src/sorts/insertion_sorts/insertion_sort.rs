use crate::create_sort;
use crate::traits::log_traits::SortLogger;

create_sort!(sort, "insertion sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 1..arr.len() {
        let key = arr[i];
        let mut j = i;
        while j > 0 && logger.cmp_gt_data(arr, j - 1, key) {
            j -= 1;
        }
        if j < i {
            logger.shift_insert(arr, i, j, key);
        }
    }
}

/// Public dyn-dispatch entry point used by `insertion_sorts::fn_sort`.
pub fn sort_dyn(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
    sort(arr, logger);
}
