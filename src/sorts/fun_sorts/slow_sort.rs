const MAX_SIZE: usize = 150;
const BIG_O: &str = "O(N^3)";
const NAME: &str = "slow sort";

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
    let len = arr.len();

    let mid = arr.len() / 2;
    sort(&mut arr[..mid], logger);
    sort(&mut arr[mid..], logger);
    logger.cond_swap_gt(arr, mid - 1, len - 1);
    sort(&mut arr[..len - 1], logger);
}
