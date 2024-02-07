const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N^log^2(N))";
const NAME: &str = "circle_sort";

use crate::traits;
use crate::traits::*;
use std::marker::PhantomData;

pub struct SortImp<T: Ord + Copy, U: traits::log_traits::SortLogger<T>> {
    _markers: (PhantomData<T>, PhantomData<U>),
}

impl<T: Ord + Copy, U: traits::log_traits::SortLogger<T>> traits::sort_traits::SortAlgo<T, U>
    for SortImp<T, U>
{
    fn max_size() -> usize {
        MAX_SIZE
    }
    fn big_o() -> &'static str {
        BIG_O
    }
    fn sort(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name() -> &'static str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    while circle_sort_rec(arr, 0, arr.len() - 1, logger) {}
}

fn circle_sort_rec<T: Ord + Copy, U: log_traits::SortLogger<T>>(
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
    let second_half = circle_sort_rec(arr, mid + 1, end, logger);

    let (mut start_tmp, mut end_tmp) = (start, end);

    while start_tmp < end_tmp {
        if logger.cond_swap_lt(arr, end_tmp, start_tmp) {
            swapped = true;
        }

        start_tmp += 1;
        end_tmp -= 1;
    }

    if start_tmp == end_tmp && logger.cond_swap_lt(arr, start_tmp + 1, start_tmp) {
        swapped = true;
    }
    let first_half = circle_sort_rec(arr, start, mid, logger);

    swapped || first_half || second_half
}
