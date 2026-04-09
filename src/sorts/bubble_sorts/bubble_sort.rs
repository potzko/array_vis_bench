use crate::create_sort;
use crate::traits::log_traits::SortLogger;

create_sort!(sort, "bubble sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for ii in 0..arr.len() {
        for i in 1..arr.len() - ii {
            logger.cond_swap_lt(arr, i, i - 1);
        }
    }
}

pub fn sort_dyn(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
    sort(arr, logger);
}
