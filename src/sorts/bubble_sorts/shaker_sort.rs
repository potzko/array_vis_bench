const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shaker sort";

use std::sync::BarrierWaitResult;

use crate::traits;
pub struct BubbleSort {}

impl traits::sort_traits::SortAlgo for BubbleSort {
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
    let len = end - start;
    for ii in start..len / 2 + start {
        let mut swaps = false;
        for i in ii + 1..end - (ii - start) {
            swaps |= logger.cond_swap_lt(arr, i, i - 1);
        }
        if !swaps {
            break;
        }
        for i in (ii + 1..end - (ii - start)).rev() {
            swaps |= logger.cond_swap_lt(arr, i, i - 1);
        }
        if !swaps {
            break;
        }
    }
}
