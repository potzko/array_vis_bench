const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "bad_heap";

use crate::traits;
use traits::log_traits::SortLog;
pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
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

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use crate::traits::log_traits::SortLog::*;

    first_heapify(arr, start, end, logger);
    for sorted_index in (1 + start..end).rev() {
        logger.log(Swap {
            name: &arr as *const _ as usize,
            ind_a: start,
            ind_b: sorted_index,
        });
        arr.swap(0, sorted_index);
        heapify(arr, start, sorted_index - 1, logger);
    }
    use traits::sort_traits::SortAlgo;
    crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(
        arr,
        start,
        start + std::cmp::min(end - start, 3),
        logger,
    );
}

fn heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use traits::log_traits::SortLog::*;

    let mut ind = start;
    let mut left = ind + 1;
    let mut right = ind + 2;

    if right < end {
        logger.log(Cmp {
            name: &arr as *const _ as usize,
            ind_a: right,
            ind_b: left,
            result: arr[right] < arr[left],
        });
        if arr[right] > arr[left] {
            left = right;
        }
    }

    while left < end {
        logger.log(Cmp {
            name: &arr as *const _ as usize,
            ind_a: left,
            ind_b: ind,
            result: arr[left] < arr[ind],
        });
        if arr[left] > arr[ind] {
            logger.log(Swap {
                name: &arr as *const _ as usize,
                ind_a: ind,
                ind_b: left,
            });
            arr.swap(ind, left);
            ind = left;
            left = ind + 1;
            right = ind + 2;
            if right < end {
                logger.log(Cmp {
                    name: &arr as *const _ as usize,
                    ind_a: right,
                    ind_b: left,
                    result: arr[right] < arr[left],
                });
                if arr[right] > arr[left] {
                    left = right;
                }
            }
        } else {
            break;
        }
    }
}

fn deep_heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if end <= start {
        return;
    }
    let left = start + 1;
    let right = start + 2;
    deep_heapify(arr, right, end, logger);
    deep_heapify(arr, left, end, logger);
    heapify(arr, start, end, logger);
}

fn first_heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    for start in (start..start + (end - start)).rev() {
        heapify(arr, start, end - 1, logger);
    }
}
