use crate::create_sort;

create_sort!(sort, "bad heap sort alt", "O(N^2)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 0..arr.len() {
        for j in 0..arr.len() {
            if logger.cmp_lt(arr, j, i) {
                logger.swap(arr, i, j);
            }
        }
    }
}
