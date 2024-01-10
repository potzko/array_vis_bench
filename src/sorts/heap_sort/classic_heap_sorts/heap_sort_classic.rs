const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "heap sort";

use crate::traits;
use crate::traits::log_traits::SortLogger;
pub struct HeapSort {}

impl traits::sort_traits::SortAlgo for HeapSort {
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

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    first_heapify(arr, logger);

    for i in (1..arr.len()).rev() {
        logger.swap(arr, 0, i);
        heapify(arr, 0, i, logger);
    }
}

fn first_heapify<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for start in (0..arr.len() / 2).rev() {
        heapify(arr, start, arr.len(), logger);
    }
}

fn heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut ind = start;
    let mut left = (ind << 1) | 1;
    let mut right = (ind + 1) << 1;

    if right < end && logger.cmp_gt(arr, right, left) {
        left = right;
    }

    while left < end && logger.cmp_gt(arr, left, ind) {
        logger.swap(arr, ind, left);
        ind = left;
        left = (ind << 1) | 1;
        right = (ind + 1) << 1;
        if right < end && logger.cmp_gt(arr, right, left) {
            left = right;
        }
    }
}
