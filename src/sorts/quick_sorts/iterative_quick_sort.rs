const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left right pointers moving pivot";

use std::collections::VecDeque;

use crate::traits::{self, SortAlgo};
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

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    let mut low = 0;
    let mut high = arr.len() - 1;
    while low < high - 1 {
        if logger.cond_swap_le(arr, low + 1, low) {
            low += 1;
        } else {
            logger.swap(arr, low + 1, high);
            high -= 1;
        }
    }
    logger.cond_swap_lt(arr, high, low);
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut inds: VecDeque<(usize, usize)> = VecDeque::with_capacity(arr.len() / 32);
    inds.push_front((0, arr.len()));
    if arr.len() < 2 {
        return;
    }
    while let Some((start, end)) = inds.pop_back() {
        let array_slice = &mut arr[start..end];
        if array_slice.len() > 32 {
            let part = start + partition(array_slice, logger);
            inds.push_front((start, part));
            inds.push_front((part, end));
        }
    }
    type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
    SmallSort::sort(arr, logger)
}
