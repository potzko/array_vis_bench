const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left right pointers optimised";

use crate::traits::{self, sort_traits::SortAlgo};
pub struct QuickSort {}

impl traits::sort_traits::SortAlgo for QuickSort {
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
impl Default for QuickSort {
    fn default() -> Self {
        QuickSort {}
    }
}
impl Debug for QuickSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    assert!(arr.len() >= 3);
    // Find the median of three to use as the pivot
    logger.cond_swap_lt(arr, 0, arr.len() - 1);

    logger.cond_swap_lt(arr, 1, 0);

    logger.cond_swap_lt(arr, 0, arr.len() - 1);

    let mut low = 1;
    let mut high = arr.len() - 1;
    while low >= arr.len() && logger.cmp_le(arr, 0, low) {
        low += 1;
    }
    if low == arr.len() {
        //to do stuff
        logger.swap(arr, 0, arr.len() - 1);
        return arr.len() - 1;
    }
    while logger.cmp_gt(arr, high, 0) {
        high -= 1
    }
    // Continue until the pointers cross
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
    if arr.len() < 256 {
        let small_sort = crate::sorts::shell_sorts::classic_shell_sorts::shell_optimized_256_elements::ShellSort {};
        small_sort.sort(arr, logger);
        //crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger);
        return;
    }
    let partition_index = partition(arr, logger);
    sort(&mut arr[0..partition_index], logger);
    sort(&mut arr[partition_index + 1..], logger);
}
