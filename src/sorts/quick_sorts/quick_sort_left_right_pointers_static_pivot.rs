const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

use crate::traits;
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
    let mut low = 1;
    let mut high = arr.len() - 1;
    while low >= arr.len() && logger.cmp_le(arr, 0, low) {
        low += 1;
    }
    if low == arr.len() {
        logger.swap(arr, 0, arr.len() - 1);
        return arr.len() - 1;
    }
    while logger.cmp_gt(arr, high, 0) {
        high -= 1
    }
    while low <= high {
        logger.swap(arr, low, high);
        while low <= high && !logger.cmp_gt(arr, low, 0) {
            low += 1;
        }
        while logger.cmp_gt(arr, high, 0) {
            high -= 1;
        }
    }

    logger.swap(arr, 0, high);
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }

    let partition_index = partition(arr, logger);
    sort(&mut arr[..partition_index], logger);
    sort(&mut arr[partition_index + 1..], logger);
}
