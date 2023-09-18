const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

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
    if end - start >= 2 {
        // Logging the first comparison
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: end - 1,
            ind_b: start,
            result: arr[end - 1] < arr[start],
        });

        if arr[end - 1] < arr[start] {
            // Logging the first swap
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: start,
                ind_b: end - 1,
            });
            arr.swap(start, end - 1);
        }

        // Logging the second comparison
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: end - 1,
            ind_b: end - 2,
            result: arr[end - 1] > arr[end - 2],
        });

        if arr[end - 1] > arr[end - 2] {
            // Logging the second swap
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: end - 2,
                ind_b: end - 1,
            });
            arr.swap(end - 2, end - 1);
        }

        // Logging the third comparison
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: end - 1,
            ind_b: start,
            result: arr[end - 1] < arr[start],
        });

        if arr[end - 1] < arr[start] {
            // Logging the third swap
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: start,
                ind_b: end - 1,
            });
            arr.swap(start, end - 1);
        }
    }

    let pivot = arr[end - 1];
    let mut small = start;
    for i in start..end - 1 {
        logger.log(SortLog::CmpSingle {
            name: &arr as *const _ as usize,
            ind_a: i,
            result: arr[i] < pivot,
        });
        if arr[i] < pivot {
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: i,
                ind_b: small,
            });
            arr.swap(i, small);
            small += 1;
        }
    }
    logger.log(SortLog::Swap {
        name: &arr as *const _ as usize,
        ind_a: small,
        ind_b: end - 1,
    });
    arr.swap(small, end - 1);
    small
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if end - start < 16 {
        crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger);
        return;
    }
    logger.log(SortLog::Mark(format!("sorting {} to {}", start, end)));

    let partition_index = partition(arr, start, end, logger);
    sort(arr, start, partition_index, logger);
    sort(arr, partition_index + 1, end, logger);
}
