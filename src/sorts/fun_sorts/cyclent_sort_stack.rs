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
    use std::collections::VecDeque;
    let mut stack: VecDeque<usize> =
        VecDeque::with_capacity((arr.len() as f64).log2() as usize * 4);
    stack.push_front(partition(arr, start, end, logger));
    for i in start..end - 1 {
        while stack.is_empty() || *stack.front().unwrap() != i {
            stack.push_front(partition(arr, i, *stack.front().unwrap_or(&end), logger))
        }
        stack.pop_front();
    }
}
