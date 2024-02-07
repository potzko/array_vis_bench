const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

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
    assert!(arr.len() >= 4);
    if logger.cmp_lt(arr, arr.len() - 1, arr.len() - 2) {
        logger.swap(arr, arr.len() - 1, arr.len() - 2);
    }
    if logger.cmp_lt(arr, arr.len() - 1, arr.len() - 3) {
        logger.swap(arr, arr.len() - 1, arr.len() - 3);
    }
    if logger.cmp_lt(arr, arr.len() - 2, arr.len() - 1) {
        logger.swap(arr, arr.len() - 2, arr.len() - 1);
    }
    let pivot = arr[arr.len() - 1];

    let mut small = 0;
    for i in 0..arr.len() - 1 {
        if logger.cmp_le_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, arr.len() - 1);
    small
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 32 {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }

    let partition_index = partition(arr, logger);
    sort(&mut arr[..partition_index], logger);
    sort(&mut arr[partition_index + 1..], logger);
}
