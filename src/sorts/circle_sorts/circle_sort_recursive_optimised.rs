const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N^log^2(N))";
const NAME: &str = "circle_sort";

use crate::traits;
use crate::traits::*;
use sort_traits::SortAlgo;
use std::fmt::Debug;

pub struct CircleSort {}

impl sort_traits::SortAlgo for CircleSort {
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
impl Default for CircleSort {
    fn default() -> Self {
        CircleSort {}
    }
}
impl Debug for CircleSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    for _ in 0..(((arr.len()) as f64).log2() as usize) >> 1 {
        circle_sort_rec(arr, 0, arr.len() - 1, logger);
    }
    let small_sort = crate::sorts::insertion_sorts::insertion_sort::InsertionSort {};
    small_sort.sort(arr, logger);
}

fn circle_sort_rec<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    if start == end {
        return;
    }

    let (mut start_tmp, mut end_tmp) = (start, end);

    while start_tmp < end_tmp {
        logger.cond_swap_lt(arr, end_tmp, start_tmp);

        start_tmp += 1;
        end_tmp -= 1;
    }

    if start_tmp == end_tmp {
        logger.cond_swap_lt(arr, end_tmp + 1, start_tmp);
    }

    let mid = start + (end - start) / 2;
    circle_sort_rec(arr, start, mid, logger);
    circle_sort_rec(arr, mid + 1, end, logger);
}
