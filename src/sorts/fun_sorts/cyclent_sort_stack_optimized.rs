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
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    if arr.len() > 1024 {
        logger.cond_swap_lt(arr, 0, 2);
        logger.cond_swap_lt(arr, 1, 0);
        logger.cond_swap_lt(arr, 0, 2);
        if arr.len() > 2048 {
            logger.cond_swap_lt(arr, 3, 5);
            logger.cond_swap_lt(arr, 4, 3);
            logger.cond_swap_lt(arr, 3, 5);
            logger.cond_swap_lt(arr, 6, 8);
            logger.cond_swap_lt(arr, 7, 6);
            logger.cond_swap_lt(arr, 6, 8);
            logger.cond_swap_lt(arr, 0, 6);
            logger.cond_swap_lt(arr, 3, 0);
            logger.cond_swap_lt(arr, 0, 6);
        }
    }
    let pivot = arr[0];
    let mut low = 0;
    let mut high = arr.len() - 1;
    while high > 0 && logger.cmp_ge_data(arr, high, pivot) {
        high -= 1
    }
    if high == 0 {
        return 0;
    }
    while low >= arr.len() && logger.cmp_lt_data(arr, 0, pivot) {
        low += 1;
    }
    while low <= high {
        logger.swap(arr, low, high);
        // Increment low pointer while the element at low is less than or equal to the pivot
        while low <= high && !logger.cmp_gt(arr, low, 0) {
            low += 1;
        }

        // Decrement high pointer while the element at high is greater than the pivot
        while logger.cmp_gt(arr, high, 0) {
            high -= 1;
        }
    }

    // Position the pivot correctly by swapping it with the element at 'high'
    logger.swap(arr, 0, high);
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let len = arr.len();
    use crate::traits::sort_traits::SortAlgo;
    let mut stack: Vec<usize> = Vec::with_capacity((arr.len() as f64).log2() as usize * 8);
    let mut i = 0;
    while i < arr.len() {
        while stack.is_empty() || *stack.last().unwrap() >= i + 256 {
            stack.push(i + partition(&mut arr[i..*stack.last().unwrap_or(&len)], logger))
        }
        super::super::shell_sorts::classic_shell_sorts::shell_optimized_256_elements::ShellSort::sort(
            &mut arr[i..*stack.last().unwrap_or(&len)],
            logger,
        );
        i = stack.pop().unwrap_or(i);
        i += 1
    }
    //super::super::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger)
}
