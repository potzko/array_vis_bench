const MAX_SIZE: usize = 150;
const BIG_O: &str = "O(N^3)";
const NAME: &str = "slow sort";

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
    let len = end - start;
    let mid = start + len / 2;
    sort(arr, start, mid, logger);
    sort(arr, mid, end, logger);
    logger.cond_swap_gt(arr, mid - 1, end - 1);
    sort(arr, start, end - 1, logger);
}
