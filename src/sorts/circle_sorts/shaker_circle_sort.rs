const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N^log^2(N))";
const NAME: &str = "circle_sort";

use crate::traits;
use crate::traits::*;
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
    while circle_sort_rec_inc(arr, 0, arr.len() - 1, logger) {}
}

fn circle_sort_rec_dec<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    let mut swapped = false;

    if start == end {
        return false;
    }

    let (mut start_tmp, mut end_tmp) = (start, end);

    while start_tmp < end_tmp {
        if logger.cond_swap_lt(arr, end_tmp, start_tmp) {
            swapped = true;
        }
        start_tmp += 1;
        end_tmp -= 1;
    }

    if start_tmp == end_tmp && logger.cond_swap_lt(arr, end_tmp + 1, start_tmp) {
        swapped = true;
    }

    let mid = start + (end - start) / 2;
    let first_half = circle_sort_rec_inc(arr, start, mid, logger);
    let second_half = circle_sort_rec_inc(arr, mid + 1, end, logger);

    swapped || first_half || second_half
}

fn circle_sort_rec_inc<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    let mut swapped = false;

    if start == end {
        return false;
    }

    let mid = start + (end - start) / 2;
    let first_half = circle_sort_rec_dec(arr, start, mid, logger);
    let second_half = circle_sort_rec_dec(arr, mid + 1, end, logger);
    let (mut start_tmp, mut end_tmp) = (start, end);

    while start_tmp < end_tmp {
        if logger.cond_swap_lt(arr, end_tmp, start_tmp) {
            swapped = true;
        }
        start_tmp += 1;
        end_tmp -= 1;
    }

    if start_tmp == end_tmp && logger.cond_swap_lt(arr, end_tmp + 1, start_tmp) {
        swapped = true;
    }

    swapped || first_half || second_half
}
