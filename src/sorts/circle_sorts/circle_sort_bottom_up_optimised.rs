const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N^log^2(N))";
const NAME: &str = "circle_sort";

use crate::traits;
use crate::traits::*;
use sort_traits::SortAlgo;

pub struct CircleSort {}

impl sort_traits::SortAlgo for CircleSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &'static str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        &self,
        arr: &mut [T],
        logger: &mut U,
    ) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &'static str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    for _ in 0..(((arr.len()) as f64).log2() as usize) >> 1 {
        if !circle_sort_iteration(arr, logger) {
            break;
        }
    }
    let small_sort = crate::sorts::insertion_sorts::insertion_sort::InsertionSort {};
    small_sort.sort(arr, logger);
}

fn circle_sort_iteration<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],

    logger: &mut U,
) -> bool {
    let mut swapped = false;

    let mut iter = get_last_bit(arr.len());
    while iter > 1 {
        for i in (0..arr.len()).step_by(iter) {
            let mut ind_left = i;
            let mut ind_right = i + iter - 1;
            if ind_right >= arr.len() {
                let tmp = ind_right - arr.len() + 1;
                ind_left += tmp;
                ind_right -= tmp;
            }

            while ind_left < ind_right {
                if logger.cond_swap_lt(arr, ind_right, ind_left) {
                    swapped = true
                }
                ind_left += 1;
                ind_right -= 1;
            }
        }
        iter /= 2;
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
