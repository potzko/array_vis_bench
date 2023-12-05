const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "odd-even bubble sort";

use crate::traits;

pub struct BubbleSort {}

impl traits::sort_traits::SortAlgo for BubbleSort {
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
    if arr.len() < 2 {
        return;
    }
    let mut sorted: bool = false;
    while !sorted {
        sorted = true;
        logger.mark_mssg("parity = 0");
        for ii in (2..arr.len()).step_by(2) {
            if logger.cond_swap_lt(arr, ii, ii - 1) {
                sorted = false;
            }
        }
        logger.mark_mssg("parity = 1");
        for ii in (1..arr.len()).step_by(2) {
            if logger.cond_swap_lt(arr, ii, ii - 1) {
                sorted = false;
            }
        }
    }
}
