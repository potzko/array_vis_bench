use crate::create_sort;

create_sort!(sort, "cyclent sort", "O(N^3?)", false);

fn partition<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    let pivot = arr[0];
    let mut low = 0;
    let mut high = arr.len() - 1;
    while high > 0 && logger.cmp_ge_data(arr, high, pivot) {
        high -= 1
    }
    if high == 0 {
        return 0;
    }
    while low >= arr.len() && logger.cmp_lt_data(arr, 0, pivot) {
        low += 1;
    }
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
    for i in 0..arr.len() {
        let mut tmp = arr.len();
        while tmp != i {
            tmp = partition(&mut arr[i..tmp], logger) + i;
        }
    }
}
