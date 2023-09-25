const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

use crate::traits;
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
    let mut low = start + 1;
    let mut high = end - 1;
    while low >= end && logger.cmp_le(arr, start, low) {
        low += 1;
    }
    if low == end {
        logger.swap(arr, start, end - 1);
        return end - 1;
    }
    while logger.cmp_gt(arr, high, start) {
        high -= 1
    }
    while low <= high {
        logger.swap(arr, low, high);
        while low <= high && !logger.cmp_gt(arr, low, start) {
            low += 1;
        }
        while logger.cmp_gt(arr, high, start) {
            high -= 1;
        }
    }

    logger.swap(arr, start, high);
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if end - start < 2 {
        return;
    }
    logger.mark(format!("sorting {} to {}", start, end));

    let partition_index = partition(arr, start, end, logger);
    sort(arr, start, partition_index, logger);
    sort(arr, partition_index + 1, end, logger);
}
