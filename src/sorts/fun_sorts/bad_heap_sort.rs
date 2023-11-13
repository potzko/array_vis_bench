const MAX_SIZE: usize = 500;
const BIG_O: &str = "O(N^?)";
const NAME: &str = "bad_heap";

use crate::traits;
pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
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
impl Default for FunSort {
    fn default() -> Self {
        FunSort {}
    }
}
impl Debug for FunSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    sort_rec::<T, U>(arr, 0, arr.len(), logger);
}

fn sort_rec<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if end - start < 2 {
        return;
    }

    let left = start + 1;
    let right = start * 2 + 1;

    if right < end && logger.cond_swap_lt(arr, right, left) {
        sort_rec(arr, right, end, logger);
    }

    sort_rec(arr, left, end, logger);

    if logger.cond_swap_lt(arr, left, start) {
        sort_rec(arr, start, end, logger);
    }
}
