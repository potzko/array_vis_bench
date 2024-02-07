const MAX_SIZE: usize = 1000;
const BIG_O: &str = "O(N^(logn))";
const NAME: &str = "stooge sort";

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
    logger.cond_swap_lt(arr, arr.len() - 1, 0);
    if arr.len() >= 3 {
        let len = arr.len();
        let third = arr.len() / 3;
        sort(&mut arr[..len - third], logger);
        sort(&mut arr[third..], logger);
        sort(&mut arr[..len - third], logger);
    }
}
