const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

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
    assert!(arr.len() >= 4);
    if logger.cmp_lt(arr, arr.len() - 1, arr.len() - 2) {
        logger.swap(arr, arr.len() - 1, arr.len() - 2);
    }
    if logger.cmp_lt(arr, arr.len() - 1, arr.len() - 3) {
        logger.swap(arr, arr.len() - 1, arr.len() - 3);
    }
    if logger.cmp_lt(arr, arr.len() - 2, arr.len() - 1) {
        logger.swap(arr, arr.len() - 2, arr.len() - 1);
    }
    let pivot = arr[arr.len() - 1];

    let mut small = 0;
    for i in 0..arr.len() - 1 {
        if logger.cmp_le_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, arr.len() - 1);
    small
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 64 {
        let small_sort = crate::sorts::shell_sorts::classic_shell_sorts::shell_optimized_256_elements::ShellSort {};
        small_sort.sort(arr, logger);
        return;
    }

    let partition_index = partition(arr, logger);
    sort(&mut arr[..partition_index], logger);
    sort(&mut arr[partition_index + 1..], logger);
}
