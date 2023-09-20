const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "odd-even bubble sort";

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
    if end - start < 2 {
        return;
    }
    let mut sorted: bool = false;
    while !sorted {
        sorted = true;
        logger.mark_mssg("parity = 0");
        for ii in (start + 2..end).step_by(2) {
            if logger.cond_swap_lt(arr, ii, ii - 1) {
                sorted = false;
            }
        }
        logger.mark_mssg("parity = 1");
        for ii in (start + 1..end).step_by(2) {
            if logger.cond_swap_lt(arr, ii, ii - 1) {
                sorted = false;
            }
        }
    }
}
