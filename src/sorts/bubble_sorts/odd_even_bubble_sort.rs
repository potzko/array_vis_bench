use crate::create_sort;

create_sort!(sort, "odd-even bubble sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: ?Sized + crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let mut sorted = false;
    while !sorted {
        sorted = true;
        for i in (1..arr.len()).step_by(2) {
            if logger.cond_swap_lt(arr, i, i - 1) {
                sorted = false;
            }
        }
        for i in (2..arr.len()).step_by(2) {
            if logger.cond_swap_lt(arr, i, i - 1) {
                sorted = false;
            }
        }
    }
}

pub fn sort_dyn(arr: &mut [usize], logger: &mut dyn crate::traits::log_traits::SortLogger<usize>) {
    sort(arr, logger);
}
