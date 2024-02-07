const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "merge sort";

use super::utils::merge_inplace;
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
    let mut tmp = logger.copy_aux_arr_t(arr);
    merge_sort(arr, &mut tmp, logger);
}

fn merge_sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    target: &mut [T],
    logger: &mut U,
) {
    let mut gap = 1;
    let mut i;
    while gap < arr.len() {
        i = 0;
        while i < arr.len() {
            let end = std::cmp::min(i + 2 * gap, arr.len());
            let mid = std::cmp::min(i + gap, arr.len());
            merge_inplace(&arr[i..mid], &arr[mid..end], &mut target[i..end], logger);
            i += gap * 2;
        }

        i = 0;
        gap *= 2;
        while i < arr.len() {
            let end = std::cmp::min(i + 2 * gap, arr.len());
            let mid = std::cmp::min(i + gap, arr.len());
            merge_inplace(&target[i..mid], &target[mid..end], &mut arr[i..end], logger);
            i += gap * 2;
        }
        gap *= 2;
    }
}
