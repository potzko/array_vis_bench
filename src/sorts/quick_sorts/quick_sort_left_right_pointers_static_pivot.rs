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
    let pivot = arr[end - 1]; // Taking the last element as pivot
    let mut i = start; // Index of the smaller element

    for ii in start..end - 1 {
        // If the current element is smaller than or equal to the pivot, move it to the small portion of the array
        if logger.cmp_le_data(arr, ii, pivot) {
            logger.swap(arr, i, ii);
            i += 1;
        }
    }
    logger.swap(arr, i, end - 1);
    i
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
