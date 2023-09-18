const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left right pointers moving pivot";

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
    let mut low = start;
    let mut high = end - 1;
    while low < high - 1 {
        logger.log(SortLog::Cmp {
            name: &arr as *const _ as usize,
            ind_a: low + 1,
            ind_b: low,
            result: arr[low + 1] <= arr[low],
        });
        if arr[low + 1] <= arr[low] {
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: low,
                ind_b: low + 1,
            });
            arr.swap(low, low + 1);
            low += 1;
        } else {
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: low + 1,
                ind_b: high,
            });
            arr.swap(low + 1, high);
            high -= 1;
        }
    }
    logger.log(SortLog::Cmp {
        name: &arr as *const _ as usize,
        ind_a: high,
        ind_b: low,
        result: arr[high] < arr[low],
    });
    if arr[high] < arr[low] {
        logger.log(SortLog::Swap {
            name: &arr as *const _ as usize,
            ind_a: low,
            ind_b: high,
        });
        arr.swap(low, high);
    }
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
    let partition_index = partition(arr, start, end, logger);
    sort(arr, start, partition_index, logger);
    sort(arr, partition_index, end, logger);
}
