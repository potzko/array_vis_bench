const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "cycle sort";

use crate::traits::{self, log_traits::SortLogger};
use std::mem::replace;

pub struct CycleSort {}

impl traits::sort_traits::SortAlgo for CycleSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) {
        sort::<T, U>(arr, start, end, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], start: usize, end: usize, logger: &mut U) {
    for i in start..end {
        let mut target: T = arr[i];
        let mut lower = get_lower(arr, i + 1, end, arr[i], logger) + i;
        while lower != i {
            logger.log(traits::log_traits::SortLog::WriteOutOfArr {
                name: 0,
                ind: lower,
                data: target,
            });
            target = replace(&mut arr[lower], target);
            lower = get_lower(arr, i + 1, end, target, logger) + i;
        }
        logger.log(traits::log_traits::SortLog::WriteOutOfArr {
            name: 0,
            ind: i,
            data: target,
        });
        arr[i] = target;
    }
}

fn get_lower<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    target: T,
    logger: &mut U,
) -> usize {
    use crate::traits::log_traits::SortLog::*;
    let mut ret = 0;
    for i in start..end {
        logger.log(CmpOuterData {
            name: 0,
            ind: i,
            data: target,
            result: arr[i] < target,
        });
        if arr[i] < target {
            ret += 1
        }
    }
    ret
}
