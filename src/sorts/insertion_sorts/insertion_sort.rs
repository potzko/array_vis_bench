use crate::create_sort;

create_sort!(sort, "insertion sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: ?Sized + crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 1..arr.len() {
        for ii in (1..=i).rev() {
            if !logger.cond_swap_lt(arr, ii, ii - 1) {
                break;
            }
        }
    }
}

/// Public dyn-dispatch entry point used by `insertion_sorts::fn_sort`.
pub fn sort_dyn(arr: &mut [usize], logger: &mut dyn crate::traits::log_traits::SortLogger<usize>) {
    sort(arr, logger);
}
