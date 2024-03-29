const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N^log^2(N))";
const NAME: &str = "circle_sort";

use crate::traits;
use crate::traits::*;
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

fn sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    while circle_sort_iteration_increasing(arr, logger)
        && circle_sort_iteration_decreasing(arr, logger)
    {}
}

fn circle_sort_iteration_increasing<T: Ord + Copy, U: log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let mut swapped = false;
    let max = get_last_bit(arr.len());
    let mut iter = 2;
    while iter <= max {
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
        iter *= 2;
    }
    swapped
}
fn circle_sort_iteration_decreasing<T: Ord + Copy, U: log_traits::SortLogger<T>>(
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
