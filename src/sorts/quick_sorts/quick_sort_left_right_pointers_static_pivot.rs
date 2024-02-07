const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick sort left left pointers";

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

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],

    logger: &mut U,
) -> usize {
    let mut low = 1;
    let mut high = arr.len() - 1;
    while low >= arr.len() && logger.cmp_le(arr, 0, low) {
        low += 1;
    }
    if low == arr.len() {
        logger.swap(arr, 0, arr.len() - 1);
        return arr.len() - 1;
    }
    while logger.cmp_gt(arr, high, 0) {
        high -= 1
    }
    while low <= high {
        logger.swap(arr, low, high);
        while low <= high && !logger.cmp_gt(arr, low, 0) {
            low += 1;
        }
        while logger.cmp_gt(arr, high, 0) {
            high -= 1;
        }
    }

    logger.swap(arr, 0, high);
    high
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }

    let partition_index = partition(arr, logger);
    sort(&mut arr[..partition_index], logger);
    sort(&mut arr[partition_index + 1..], logger);
}
