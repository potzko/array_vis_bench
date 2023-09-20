const MAX_SIZE: usize = 10000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "comb sort";

use crate::traits;

pub struct CombSort {}

impl traits::sort_traits::SortAlgo for CombSort {
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

const SHRINK_FACTOR: f32 = 1.3;

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut jump = end - start;
    let mut swap_flag = true;
    while swap_flag {
        swap_flag = false;
        jump = std::cmp::max(1, ((jump as f32) / SHRINK_FACTOR) as usize);
        for i in jump..end {
            if logger.cond_swap_lt(arr, i, i - jump) {
                swap_flag = true
            }
        }
    }
}
