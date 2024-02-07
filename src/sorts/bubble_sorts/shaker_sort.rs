const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shaker sort";

use std::marker::PhantomData;

use crate::traits;

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
    for ii in 0..arr.len() / 2 {
        let mut swaps = false;
        for i in ii + 1..arr.len() - ii {
            swaps |= logger.cond_swap_lt(arr, i, i - 1);
        }
        if !swaps {
            break;
        }
        for i in (ii + 1..arr.len() - ii).rev() {
            swaps |= logger.cond_swap_lt(arr, i, i - 1);
        }
        if !swaps {
            break;
        }
    }
}
