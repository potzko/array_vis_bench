const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N^log^2(N))";
const NAME: &str = "circle_sort";

use crate::traits::*;
use sort_traits::SortAlgo;
pub struct CircleSort {}

impl sort_traits::SortAlgo for CircleSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(
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
    for _ in 0..(((end - start) as f64).log2() as usize) >> 1 {
        circle_sort_iteration(arr, start, end, logger);
    }
    crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger);
}

fn circle_sort_iteration<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut iter = get_last_bit(end - start);
    while iter > 1 {
        for i in (start..end).step_by(iter) {
            let mut ind_left = i;
            let mut ind_right = i + iter - 1;
            if ind_right >= end {
                let tmp = ind_right - end + 1;
                ind_left += tmp;
                ind_right -= tmp;
            }

            while ind_left < ind_right && ind_left >= start {
                logger.cond_swap_lt(arr, ind_right, ind_left);
                ind_left += 1;
                ind_right -= 1;
            }
        }
        iter /= 2;
    }
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
