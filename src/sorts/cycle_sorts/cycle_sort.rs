const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "cycle sort";

use crate::traits::{self, log_traits::SortLogger};

pub struct CycleSort {}

impl traits::sort_traits::SortAlgo for CycleSort {
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

fn sort<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 0..arr.len() {
        let mut target: T = arr[i];
        let mut lower = get_lower(&arr[i + 1..], arr[i], logger) + i;
        while lower != i {
            let tmp = arr[lower];
            logger.write_data(arr, lower, target);
            target = tmp;
            lower = get_lower(&arr[i + 1..], target, logger) + i;
        }
        logger.write_data(arr, i, target)
    }
}

fn get_lower<T: Ord + Copy, U: SortLogger<T>>(arr: &[T], target: T, logger: &mut U) -> usize {
    let mut ret = 0;
    for i in 0..arr.len() {
        if logger.cmp_lt_data(arr, i, target) {
            ret += 1
        }
    }
    ret
}
