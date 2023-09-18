const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left right pointers optimised";

use std::fmt::Debug;

use crate::traits::{self, sort_traits::SortAlgo};
use traits::log_traits::SortLog;
pub struct QuickSort {}

impl traits::sort_traits::SortAlgo for QuickSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy + Debug, U: traits::log_traits::SortLogger<T>>(
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

fn partition<T: Ord + Copy + Debug, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> usize {
    if end - start >= 3 {
        // Logging the comparison
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: start,
            ind_b: end,
            result: arr[start] < arr[end - 1],
        });

        if arr[start] < arr[end - 1] {
            // Logging the swap
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: start,
                ind_b: end - 1,
            });
            arr.swap(start, end - 1);
        }

        // Logging the comparison
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: start + 1,
            ind_b: start,
            result: arr[start + 1] < arr[start],
        });

        if arr[start + 1] < arr[start] {
            // Logging the swap
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: start,
                ind_b: start + 1,
            });
            arr.swap(start, start + 1);
        }

        // Logging the comparison
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: start,
            ind_b: end - 1,
            result: arr[start] < arr[end - 1],
        });

        if arr[start] < arr[end - 1] {
            // Logging the swap
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: start,
                ind_b: end - 1,
            });
            arr.swap(start, end - 1);
        }
    }

    let pivot = arr[start];
    let mut low = start + 1;
    let mut high = end - 1;

    loop {
        // Increment low pointer while the element at low is less than pivot
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: low,
            ind_b: start,
            result: arr[low] <= pivot,
        });
        while low <= high && arr[low] <= pivot {
            low += 1;
        }

        // Decrement high pointer while the element at high is greater than pivot
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: high,
            ind_b: start,
            result: arr[high] > pivot,
        });
        while low <= high && arr[high] > pivot {
            high -= 1;
        }

        // If the pointers haven't crossed, swap the elements
        if low < high {
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: low,
                ind_b: high,
            });
            arr.swap(low, high);
        } else {
            break;
        }
    }

    // Swap the pivot (element at start) with the element at the high pointer
    logger.log(SortLog::Swap {
        name: &arr as *const _ as usize,
        ind_a: start,
        ind_b: high,
    });
    arr.swap(start, high);

    high // Return the high pointer as the pivot position
}

fn sort<T: Ord + Copy + Debug, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    logger.log(SortLog::Mark(format!("sorting {} to {}", start, end)));

    if end - start < 16 {
        crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger);
        return;
    }
    let partition_index = partition(arr, start, end, logger);
    sort(arr, start, partition_index, logger);
    sort(arr, partition_index + 1, end, logger);
}
