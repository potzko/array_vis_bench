const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N)^2)";
const NAME: &str = "merge sort";

use super::utils::rotate_merge;
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

const SMALL_SORT_SIZE: usize = 32;
fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < SMALL_SORT_SIZE {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }

    let (left, right) = arr.split_at_mut(arr.len() / 2);
    sort(left, logger);
    sort(right, logger);

    if !logger.cmp_le(arr, arr.len() / 2 - 1, arr.len() / 2) {
        rotate_merge(arr, arr.len() / 2, logger)
    }
}
