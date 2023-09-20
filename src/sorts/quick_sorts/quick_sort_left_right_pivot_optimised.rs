const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left right pointers optimised";

use crate::traits::{self, sort_traits::SortAlgo};
pub struct QuickSort {}

impl traits::sort_traits::SortAlgo for QuickSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) {
        sort::<T, U>(arr, start, end, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> usize {
    assert!(end - start >= 3);
    // Find the median of three to use as the pivot
    logger.cond_swap_lt(arr, start, end - 1);

    logger.cond_swap_lt(arr, start + 1, start);

    logger.cond_swap_lt(arr, start, end - 1);

    let mut low = start + 1;
    let mut high = end - 1;
    while low >= end && logger.cmp_le(arr, start, low) {
        low += 1;
    }
    if low == end {
        //to do stuff
        logger.swap(arr, start, end - 1);
        return end - 1;
    }
    while logger.cmp_gt(arr, high, start) {
        high -= 1
    }
    // Continue until the pointers cross
    while low <= high {
        logger.swap(arr, low, high);
        // Increment low pointer while the element at low is less than or equal to the pivot
        while low <= high && !logger.cmp_gt(arr, low, start) {
            low += 1;
        }

        // Decrement high pointer while the element at high is greater than the pivot
        while logger.cmp_gt(arr, high, start) {
            high -= 1;
        }
    }

    // Position the pivot correctly by swapping it with the element at 'high'
    logger.swap(arr, start, high);
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    logger.mark(format!("sorting {} to {}", start, end));

    if end - start < 256 {
        crate::sorts::shell_sorts::classic_shell_sorts::shell_optimized_256_elements::ShellSort::sort(arr, start, end, logger);
        //crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger);
        return;
    }
    let partition_index = partition(arr, start, end, logger);
    sort(arr, start, partition_index, logger);
    sort(arr, partition_index + 1, end, logger);
}
