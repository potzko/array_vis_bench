const MAX_SIZE: usize = 1000;
const BIG_O: &str = "O(N^(logn))";
const NAME: &str = "stooge sort";

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
    logger.cond_swap_lt(arr, end - 1, start);
    if end - start >= 3 {
        let third = (end - start) / 3;
        sort(arr, start, end - third, logger);
        sort(arr, start + third, end, logger);
        sort(arr, start, end - third, logger);
    }
}
