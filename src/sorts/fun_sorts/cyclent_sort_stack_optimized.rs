const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N log n?)";
const NAME: &str = "cycln't sort";

use crate::traits;

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

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> usize {
    if start + 1024 < end {
        logger.cond_swap_lt(arr, start, start + 2);
        logger.cond_swap_lt(arr, start + 1, start);
        logger.cond_swap_lt(arr, start, start + 2);
        if start + 2048 < end {
            logger.cond_swap_lt(arr, start + 3, start + 5);
            logger.cond_swap_lt(arr, start + 4, start + 3);
            logger.cond_swap_lt(arr, start + 3, start + 5);
            logger.cond_swap_lt(arr, start + 6, start + 8);
            logger.cond_swap_lt(arr, start + 7, start + 6);
            logger.cond_swap_lt(arr, start + 6, start + 8);
            logger.cond_swap_lt(arr, start, start + 6);
            logger.cond_swap_lt(arr, start + 3, start);
            logger.cond_swap_lt(arr, start, start + 6);
        }
    }
    let pivot = arr[start];
    let mut low = start;
    let mut high = end - 1;
    while high > start && logger.cmp_ge_data(arr, high, pivot) {
        high -= 1
    }
    if high == 0 {
        return start;
    }
    while low >= end && logger.cmp_lt_data(arr, start, pivot) {
        low += 1;
    }
    while low <= high {
        logger.swap(arr, low, high);
        // Increment low pointer while the element at low is less than or equal to the pivot
        while low <= high && !logger.cmp_gt(arr, low, start) {
            low += 1;
        }

        // Decrement high pointer while the element at high is greater than the pivot
        while logger.cmp_gt(arr, high, start) {
            high -= 1;
        }
    }

    // Position the pivot correctly by swapping it with the element at 'high'
    logger.swap(arr, start, high);
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use crate::traits::sort_traits::SortAlgo;
    let mut stack: Vec<usize> = Vec::with_capacity((arr.len() as f64).log2() as usize * 8);
    let mut i = start;
    while i < end {
        while stack.is_empty() || *stack.last().unwrap() >= i + 256 {
            stack.push(partition(arr, i, *stack.last().unwrap_or(&end), logger))
        }
        super::super::shell_sorts::classic_shell_sorts::shell_optimized_256_elements::ShellSort::sort(
            arr,
            i,
            *stack.last().unwrap_or(&i),
            logger,
        );
        i = stack.pop().unwrap_or(i);
        i += 1
    }
    //super::super::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger)
}
