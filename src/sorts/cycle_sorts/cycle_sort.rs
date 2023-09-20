const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "cycle sort";

use crate::traits::{self, log_traits::SortLogger};

pub struct CycleSort {}

impl traits::sort_traits::SortAlgo for CycleSort {
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

fn sort<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], start: usize, end: usize, logger: &mut U) {
    for i in start..end {
        let mut target: T = arr[i];
        let mut lower = get_lower(arr, i + 1, end, arr[i], logger) + i;
        while lower != i {
            let tmp = arr[lower];
            logger.write_data(arr, lower, target);
            target = tmp;
            lower = get_lower(arr, i + 1, end, target, logger) + i;
        }
        logger.write_data(arr, i, target)
    }
}

fn get_lower<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    target: T,
    logger: &mut U,
) -> usize {
    let mut ret = 0;
    for i in start..end {
        if logger.cmp_lt_data(arr, i, target) {
            ret += 1
        }
    }
    ret
}
