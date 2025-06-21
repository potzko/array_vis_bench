use crate::create_sort;

create_sort!(sort, "stooge sort", "O(N^(logn))", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    logger.cond_swap_lt(arr, arr.len() - 1, 0);
    if arr.len() >= 3 {
        let len = arr.len();
        let third = arr.len() / 3;
        sort(&mut arr[..len - third], logger);
        sort(&mut arr[third..], logger);
        sort(&mut arr[..len - third], logger);
    }
}
