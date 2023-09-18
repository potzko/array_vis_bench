const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N^log^2(N))";
const NAME: &str = "circle_sort";

use crate::traits::*;
use std::fmt::Debug;
pub struct CircleSort {}

impl sort_traits::SortAlgo for CircleSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy + Debug, U: log_traits::SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) {
        circle_sort::<T, U>(arr, start, end, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn circle_sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    while circle_sort_rec(arr, start, end - 1, logger) {}
}

fn circle_sort_rec<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    use log_traits::SortLog::*;

    let mut swapped = false;

    if start == end {
        return false;
    }

    let mid = start + (end - start) / 2;
    let first_half = circle_sort_rec(arr, start, mid, logger);
    let second_half = circle_sort_rec(arr, mid + 1, end, logger);
    let (mut start_tmp, mut end_tmp) = (start, end);

    while start_tmp < end_tmp {
        logger.log(Cmp {
            name: &arr as *const _ as usize,
            ind_a: start_tmp,
            ind_b: end_tmp,
            result: arr[start_tmp] > arr[end_tmp],
        });
        if arr[start_tmp] > arr[end_tmp] {
            logger.log(Swap {
                name: &arr as *const _ as usize,
                ind_a: start_tmp,
                ind_b: end_tmp,
            });
            arr.swap(start_tmp, end_tmp);
            swapped = true;
        }
        start_tmp += 1;
        end_tmp -= 1;
    }

    if start_tmp == end_tmp && arr[start_tmp] > arr[end_tmp + 1] {
        logger.log(Swap {
            name: &arr as *const _ as usize,
            ind_a: start_tmp,
            ind_b: end_tmp + 1,
        });
        arr.swap(start_tmp, end_tmp + 1);
        swapped = true;
    }

    swapped || first_half || second_half
}
