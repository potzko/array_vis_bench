use crate::create_sort;

create_sort!(
    sort,
    "quick sort left right pointers moving pivot",
    "O(N Log(N))",
    false
);

fn partition<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    let mut low = 0;
    let mut high = arr.len() - 1;
    while low < high - 1 {
        if logger.cond_swap_le(arr, low + 1, low) {
            low += 1;
        } else {
            logger.swap(arr, low + 1, high);
            high -= 1;
        }
    }
    logger.cond_swap_lt(arr, high, low);
    high
}

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    let partition_index = partition(arr, logger);
    sort(&mut arr[..partition_index], logger);
    sort(&mut arr[partition_index..], logger);
}
