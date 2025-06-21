use crate::create_sort;

create_sort!(sort, "bubble sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for ii in 0..arr.len() {
        for i in 1..arr.len() - ii {
            logger.cond_swap_le(arr, i, i - 1);
        }
    }
}
