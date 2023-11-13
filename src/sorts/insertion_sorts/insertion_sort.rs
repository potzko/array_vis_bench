const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "insertion sort";

use crate::traits;
pub struct InsertionSort {}

impl traits::sort_traits::SortAlgo for InsertionSort {
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
use std::fmt::Debug;
#[allow(clippy::derivable_impls)]
impl Default for InsertionSort {
    fn default() -> Self {
        InsertionSort {}
    }
}
impl Debug for InsertionSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 1..arr.len() {
        let num = arr[i];
        let mut ind = i;

        while ind > 0 && logger.cmp_gt_data(arr, ind - 1, num) {
            logger.write(arr, ind, ind - 1);
            ind -= 1;
        }
        logger.write_data(arr, ind, num);
    }
}
