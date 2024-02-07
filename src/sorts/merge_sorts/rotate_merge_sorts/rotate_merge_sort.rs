const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N)^2)";
const NAME: &str = "merge sort";

use super::utils::rotate_merge;
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
    if arr.len() < 2 {
        return;
    }

    {
        let (left, right) = arr.split_at_mut(arr.len() / 2);
        sort(left, logger);
        sort(right, logger);
    }
    rotate_merge(arr, arr.len() / 2, logger)
}
