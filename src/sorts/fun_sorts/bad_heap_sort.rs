const MAX_SIZE: usize = 500;
const BIG_O: &str = "O(N^?)";
const NAME: &str = "bad_heap";

use crate::traits;
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
