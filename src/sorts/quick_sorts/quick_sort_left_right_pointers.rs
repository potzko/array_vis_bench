const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left right pointers";

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
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger>(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> usize {
    let mut low = start;
    let mut high = end - 1;
    while low < high - 1 {
        logger.log(SortLog::Cmp {
            name: 0,
            ind_a: low + 1,
            ind_b: low,
            result: arr[low + 1] <= arr[low],
        });
        if arr[low + 1] <= arr[low] {
            logger.log(SortLog::Swap {
                name: 0,
                ind_a: low,
                ind_b: low + 1,
            });
            arr.swap(low, low + 1);
            low += 1;
        } else {
            logger.log(SortLog::Swap {
                name: 0,
                ind_a: low + 1,
                ind_b: high,
            });
            arr.swap(low + 1, high);
            high -= 1;
        }
    }
    logger.log(SortLog::Cmp {
        name: 0,
        ind_a: high,
        ind_b: low,
        result: arr[high] < arr[low],
    });
    if arr[high] < arr[low] {
        logger.log(SortLog::Swap {
            name: 0,
            ind_a: low,
            ind_b: high,
        });
        arr.swap(low, high);
    }
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger>(arr: &mut [T], logger: &mut U) {
    sort_helper(arr, 0, arr.len(), logger);
}

fn sort_helper<T: Ord + Copy, U: traits::log_traits::SortLogger>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if end - start < 2 {
        return;
    }
    let partition_index = partition(arr, start, end, logger);
    sort_helper(arr, start, partition_index, logger);
    sort_helper(arr, partition_index, end, logger);
}
