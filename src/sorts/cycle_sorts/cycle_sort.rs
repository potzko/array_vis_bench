const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "cycle sort";

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
/*
fn sort_2<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 0..arr.len() {
        let mut target: T = arr[i];
        let mut lower = get_lower(&arr[i + 1..], target, logger) + i;
        while lower != i {
            let tmp = arr[lower];
            logger.write_data(arr, lower, target);
            target = tmp;
            lower = get_lower(&arr[i + 1..], target, logger) + i;
        }
        logger.write_data(arr, i, target)
    }
}
*/

fn sort<T: Ord + Copy, U: traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 0..arr.len() {
        let mut target = arr[i];
        let mut lower = get_lower(&arr[i + 1..], target, logger) + i;
        while lower != i {
            let mut tmp = arr[lower];
            while tmp == target {
                lower += 1;
                tmp = arr[lower];
            }
            logger.write_data(arr, lower, target);
            target = tmp;
            lower = get_lower(&arr[i + 1..], target, logger) + i;
        }
        logger.write_data(arr, i, target)
    }
}

fn get_lower<T: Ord + Copy, U: traits::SortLogger<T>>(
    arr: &[T],
    target: T,
    logger: &mut U,
) -> usize {
    let mut ret = 0;
    for i in 0..arr.len() {
        if logger.cmp_lt_data(arr, i, target) {
            ret += 1
        }
    }
    ret
}
