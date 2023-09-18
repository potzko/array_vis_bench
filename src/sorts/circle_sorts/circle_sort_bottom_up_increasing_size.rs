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
    while circle_sort_iteration(arr, start, end, logger) {}
}

fn circle_sort_iteration<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    use log_traits::SortLog::*;
    let mut swapped = false;
    let max = get_last_bit(end - start);
    let mut iter = 2;
    while iter <= max {
        for i in (start..end).step_by(iter) {
            let mut ind_left = i;
            let mut ind_right = i + iter - 1;
            if ind_right >= end {
                let tmp = ind_right - end + 1;
                ind_left += tmp;
                ind_right -= tmp;
            }

            /*while  ind_right >= end {
                ind_left += 1;
                ind_right -= 1;
            }*/

            while ind_left < ind_right && ind_left >= start {
                logger.log(Cmp {
                    name: &arr as *const _ as usize,
                    ind_a: ind_left,
                    ind_b: ind_right,
                    result: arr[ind_right] < arr[ind_left],
                });
                if arr[ind_right] < arr[ind_left] {
                    logger.log(Swap {
                        name: &arr as *const _ as usize,
                        ind_a: ind_left,
                        ind_b: ind_right,
                    });
                    arr.swap(ind_left, ind_right);
                    swapped = true;
                }
                ind_left += 1;
                ind_right -= 1;
            }
        }
        iter *= 2;
    }
    swapped
}

fn get_last_bit(mut num: usize) -> usize {
    if num == 0 {
        return 0;
    }
    let mut counter = 0;
    while num > 0 {
        num >>= 1;
        counter += 1;
    }
    1 << (counter)
}
