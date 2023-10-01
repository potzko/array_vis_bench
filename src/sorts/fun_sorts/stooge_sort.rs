const MAX_SIZE: usize = 1000;
const BIG_O: &str = "O(N^(logn))";
const NAME: &str = "stooge sort";

use crate::traits;
pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    logger.cond_swap_lt(arr, arr.len() - 1, 0);
    if arr.len() >= 3 {
        let len = arr.len();
        let third = arr.len() / 3;
        sort(&mut arr[..len - third], logger);
        sort(&mut arr[third..], logger);
        sort(&mut arr[..len - third], logger);
    }
}
