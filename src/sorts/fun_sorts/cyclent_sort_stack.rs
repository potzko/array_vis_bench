const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N log n?)";
const NAME: &str = "cycln't sort";

use crate::traits;

pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &'static str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        &self,
        arr: &mut [T],
        logger: &mut U,
    ) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &'static str {
        NAME
    }
}
use std::fmt::Debug;
#[allow(clippy::derivable_impls)]
impl Default for FunSort {
    fn default() -> Self {
        FunSort {}
    }
}
impl Debug for FunSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
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
    if arr.len() < 2 {
        return;
    }
    let len = arr.len();

    use std::collections::VecDeque;
    let mut stack: VecDeque<usize> =
        VecDeque::with_capacity((arr.len() as f64).log2() as usize * 4);
    stack.push_front(partition(arr, logger));
    for i in 0..arr.len() - 1 {
        while stack.is_empty() || *stack.front().unwrap() != i {
            stack.push_front(i + partition(&mut arr[i..*stack.front().unwrap_or(&len)], logger))
        }
        stack.pop_front();
    }
}
