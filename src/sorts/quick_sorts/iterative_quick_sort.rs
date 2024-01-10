const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left right pointers moving pivot";

use std::collections::VecDeque;

use crate::traits;
use traits::SortAlgo;
pub struct QuickSort {}

impl traits::sort_traits::SortAlgo for QuickSort {
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
    let small_sort = crate::sorts::insertion_sorts::insertion_sort::InsertionSort {};
    small_sort.sort(arr, logger)
}
