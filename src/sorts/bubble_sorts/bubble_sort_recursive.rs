use crate::create_sort;

create_sort!(sort, "bubble sort recursive", "O(N^2)", true);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    for i in 1..arr.len() {
        logger.cond_swap_le(arr, i, i - 1);
    }
    let len = arr.len();
    sort(&mut arr[..len - 1], logger);
}
