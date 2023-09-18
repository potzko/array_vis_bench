const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

use crate::traits;
use traits::log_traits::SortLog;
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

    for j in start..end - 1 {
        // If the current element is smaller than or equal to the pivot
        if arr[j] <= pivot {
            logger.log(SortLog::Cmp {
                name: &arr as *const _ as usize,
                ind_a: i,
                ind_b: j,
                result: arr[j] <= pivot,
            });

            // Swap arr[i] and arr[j]
            arr.swap(i, j);
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: i,
                ind_b: j,
            });

            i += 1; // Increment the index of the smaller element
        }
    }

    // Swap arr[i] and arr[end - 1] (or pivot)
    arr.swap(i, end - 1);
    logger.log(SortLog::Swap {
        name: &arr as *const _ as usize,
        ind_a: i,
        ind_b: end - 1,
    });

    i // Return the partition index
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
    logger.log(SortLog::Mark(format!("sorting {} to {}", start, end)));

    let partition_index = partition(arr, start, end, logger);
    sort(arr, start, partition_index, logger);
    sort(arr, partition_index + 1, end, logger);
}
