const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "heap sort";

use crate::traits;
use crate::traits::log_traits::SortLogger;
pub struct HeapSort {}

const HEAP_SIZE: usize = 3;

impl traits::sort_traits::SortAlgo for HeapSort {
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
    first_heapify(arr, start, end, logger);

    for i in (start + 1..end).rev() {
        logger.swap(arr, start, i);
        heapify(arr, start, i, logger);
    }
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

    let mut ind = start;
    let mut max_child = start * HEAP_SIZE + 1;
    max_child = max_ind(arr, max_child, min(max_child + HEAP_SIZE, end), logger);

    while max_child < end {
        if logger.cond_swap_ge(arr, max_child, ind) {
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
    let mut max = start;
    for i in start + 1..end {
        if logger.cmp_le(arr, max, i) {
            max = i
        }
    }
    max
}
