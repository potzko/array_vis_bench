const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "bad_heap";

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
fn heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut ind = start;
    let mut left = ind + 1;
    let mut right = ind * 2;

    if right < end && logger.cmp_gt(arr, right, left) {
        left = right;
    }

    while left < end && logger.cmp_gt(arr, left, ind) {
        logger.swap(arr, ind, left);
        ind = left;
        left = ind + 1;
        right = ind + 2;
        if right < end && logger.cmp_gt(arr, right, left) {
            left = right;
        }
    }
}

fn first_heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    for start in (start..end).rev() {
        heapify(arr, start, end, logger);
    }
}
