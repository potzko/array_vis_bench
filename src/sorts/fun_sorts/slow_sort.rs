const MAX_SIZE: usize = 150;
const BIG_O: &str = "O(N^3)";
const NAME: &str = "slow sort";

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
