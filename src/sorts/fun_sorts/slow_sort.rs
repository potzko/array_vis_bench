use crate::create_sort;

create_sort!(sort, "slow sort", "O(N^3)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    let len = arr.len();

    let mid = arr.len() / 2;
    sort(&mut arr[..mid], logger);
    sort(&mut arr[mid..], logger);
    logger.cond_swap_gt(arr, mid - 1, len - 1);
    sort(&mut arr[..len - 1], logger);
}
