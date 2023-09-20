const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

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
    assert!(end - start >= 4);
    if logger.cmp_lt(arr, end - 1, end - 2) {
        logger.swap(arr, end - 1, end - 2);
    }
    if logger.cmp_lt(arr, end - 1, end - 3) {
        logger.swap(arr, end - 1, end - 3);
    }
    if logger.cmp_lt(arr, end - 2, end - 1) {
        logger.swap(arr, end - 2, end - 1);
    }
    let pivot = arr[end - 1];

    let mut small = start;
    for i in start..end - 1 {
        if logger.cmp_le_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, end - 1);
    small
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if end - start < 64 {
        crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger);
        return;
    }
    logger.mark(format!("sorting {} to {}", start, end));

    let partition_index = partition(arr, start, end, logger);
    sort(arr, start, partition_index, logger);
    sort(arr, partition_index + 1, end, logger);
}
