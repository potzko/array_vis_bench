const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "shell sort";

use std::fmt::Debug;

use crate::traits;
use crate::traits::log_traits::SortLogger;
use crate::traits::sort_traits::SortAlgo;
pub struct HeapSort {}

const HEAP_SIZE: usize = 256;

impl traits::sort_traits::SortAlgo for HeapSort {
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

fn sort<T: Ord + Copy + Debug, U: traits::log_traits::SortLogger<T>>(
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
    crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(
        arr,
        start,
        start + std::cmp::min(end - start, HEAP_SIZE + 1),
        logger,
    );
}

fn first_heapify<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    for start in (start..start + (end - start) / HEAP_SIZE + HEAP_SIZE).rev() {
        heapify(arr, start, end, logger);
    }
}

fn heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use std::cmp::min;
    use traits::log_traits::SortLog::*;

    let mut ind = start;
    let mut max_child = start * HEAP_SIZE + 1;
    max_child = max_ind(arr, max_child, min(max_child + HEAP_SIZE, end), logger);

    while max_child < end {
        logger.log(Cmp {
            name: &arr as *const _ as usize,
            ind_a: max_child,
            ind_b: ind,
            result: arr[max_child] < arr[ind],
        });
        if arr[max_child] > arr[ind] {
            logger.log(Swap {
                name: &arr as *const _ as usize,
                ind_a: ind,
                ind_b: max_child,
            });
            arr.swap(ind, max_child);
            ind = max_child;
            max_child = max_ind(
                arr,
                ind * HEAP_SIZE + 1,
                min(ind * HEAP_SIZE + HEAP_SIZE + 1, end),
                logger,
            );
        } else {
            break;
        }
    }
}

fn max_ind<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> usize {
    use traits::log_traits::SortLog::*;
    let mut max = start;
    for i in start + 1..end {
        logger.log(Cmp {
            name: &arr as *const _ as usize,
            ind_a: max,
            ind_b: i,
            result: arr[max] < arr[i],
        });
        if arr[max] < arr[i] {
            max = i
        }
    }
    max
}
