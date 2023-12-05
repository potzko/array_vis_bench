const MAX_SIZE: usize = 10000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "comb sort";

use crate::traits;

pub struct CombSort {}

impl traits::sort_traits::SortAlgo for CombSort {
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
const SHRINK_FACTOR: f32 = 1.3;

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut jump = arr.len();
    let mut swap_flag = true;
    while jump != 1 {
        jump = std::cmp::max(1, ((jump as f32) / SHRINK_FACTOR) as usize);
        for i in jump..arr.len() {
            logger.cond_swap_lt(arr, i, i - jump);
        }
    }
    while swap_flag {
        swap_flag = false;
        for i in 1..arr.len() {
            if logger.cond_swap_lt(arr, i, i - 1) {
                swap_flag = true
            }
        }
    }
}
