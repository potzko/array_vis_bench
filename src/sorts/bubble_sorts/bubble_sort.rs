const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "bubble sort";

use crate::traits;
use std::fmt::Debug;

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
#[allow(clippy::derivable_impls)]
impl Default for BubbleSort {
    fn default() -> Self {
        BubbleSort {}
    }
}
impl Debug for BubbleSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for ii in 0..arr.len() {
        for i in 1..arr.len() - ii {
            logger.cond_swap_le(arr, i, i - 1);
        }
    }
}
