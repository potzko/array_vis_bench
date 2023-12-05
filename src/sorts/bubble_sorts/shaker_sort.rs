const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shaker sort";

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
    for ii in 0..arr.len() / 2 {
        let mut swaps = false;
        for i in ii + 1..arr.len() - ii {
            swaps |= logger.cond_swap_lt(arr, i, i - 1);
        }
        if !swaps {
            break;
        }
        for i in (ii + 1..arr.len() - ii).rev() {
            swaps |= logger.cond_swap_lt(arr, i, i - 1);
        }
        if !swaps {
            break;
        }
    }
}
