const MAX_SIZE: usize = 500;
const BIG_O: &str = "O(N^?)";
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
    if end - start < 2 {
        return;
    }

    let left = start + 1;
    let right = start * 2 + 1;

    if right < end && logger.cond_swap_lt(arr, right, left) {
        sort(arr, right, end, logger);
    }

    sort(arr, left, end, logger);

    if logger.cond_swap_lt(arr, left, start) {
        sort(arr, start, end, logger);
    }
}
