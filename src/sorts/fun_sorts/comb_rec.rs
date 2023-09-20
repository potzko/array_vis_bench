const MAX_SIZE: usize = 1000;
const BIG_O: &str = "O(N^?)";
const NAME: &str = "comb sort";

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
    while comb_iter(arr, start, end, (end - start) / 2, logger) {
        sort(
            arr,
            start,
            start + ((end - start) as f64 * 2.0 / 3.0) as usize,
            logger,
        );
        sort(arr, start + (end - start) / 3, end, logger);
        sort(
            arr,
            start,
            start + ((end - start) as f64 * 2.0 / 3.0) as usize,
            logger,
        );
    }

    while comb_iter(arr, start, end, 1, logger) {}
}

fn comb_iter<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    jump: usize,
    logger: &mut U,
) -> bool {
    if end - start < 2 {
        return false;
    }

    let mut swap_flag = false;

    for i in start + jump..end {
        if logger.cond_swap_le(arr, i, i - jump) {
            swap_flag = true;
        }
    }
    swap_flag
}
