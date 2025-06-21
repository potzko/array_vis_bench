use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(sort, "median pivot quick sort", "O(N Log(N))", false);

fn partition<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    assert!(arr.len() >= 3);
    // Find the median of three to use as the pivot
    logger.cond_swap_lt(arr, 0, arr.len() - 1);
    logger.cond_swap_lt(arr, 1, 0);
    logger.cond_swap_lt(arr, 0, arr.len() - 1);

    let mut low = 1;
    let mut high = arr.len() - 1;
    while low < arr.len() && logger.cmp_le(arr, 0, low) {
        low += 1;
    }
    if low == arr.len() {
        logger.swap(arr, 0, arr.len() - 1);
        return arr.len() - 1;
    }
    while logger.cmp_gt(arr, high, 0) {
        high -= 1
    }
    // Continue until the pointers cross
    while low <= high {
        logger.swap(arr, low, high);
        // Increment low pointer while the element at low is less than or equal to the pivot
        while low <= high && !logger.cmp_gt(arr, low, 0) {
            low += 1;
        }

        // Decrement high pointer while the element at high is greater than the pivot
        while logger.cmp_gt(arr, high, 0) {
            high -= 1;
        }
    }

    // Position the pivot correctly by swapping it with the element at 'high'
    logger.swap(arr, 0, high);
    high
}

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 32 {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }
    let partition_index = partition(arr, logger);
    sort(&mut arr[..partition_index], logger);
    sort(&mut arr[partition_index + 1..], logger);
}
