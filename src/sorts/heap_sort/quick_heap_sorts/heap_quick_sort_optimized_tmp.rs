const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "heap sort";

use crate::traits::{self, SortAlgo};
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

const SMALL_SORT_SIZE: usize = 8;

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
    if arr.len() < SMALL_SORT_SIZE {
        SmallSort::sort(arr, logger);
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_lt(&mut arr[0..split], logger);
    first_heapify_gt(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, arr.len() - 1, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_right(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split..], logger);
    SmallSort::sort(arr, logger);
}

fn sort_left<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < SMALL_SORT_SIZE {
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_gt(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, arr.len() - 1, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_right(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split..], logger);
}

fn sort_right<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < SMALL_SORT_SIZE {
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_lt(&mut arr[0..split], logger);
    while logger.cond_swap_lt(arr, arr.len() - 1, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_right(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split..], logger);
}

fn first_heapify_lt<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let len = arr.len();
    for i in (0..len / 2).rev() {
        heapify_lt(arr, i, len, logger);
    }
}

fn first_heapify_gt<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let len = arr.len();
    for i in (0..len / 2).rev() {
        heapify_gt_right(arr, i, len, logger);
    }
}

fn heapify_lt<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut ind = start;
    let mut left = (ind << 1) | 1;
    let mut right = (ind + 1) << 1;

    if right < end && logger.cmp_gt(arr, right, left) {
        left = right;
    }

    while left < end && logger.cmp_gt(arr, left, ind) {
        logger.swap(arr, ind, left);
        ind = left;
        left = (ind << 1) | 1;
        right = (ind + 1) << 1;
        if right < end && logger.cmp_gt(arr, right, left) {
            left = right;
        }
    }
}

fn heapify_gt_right<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let len = arr.len();
    let mut ind = start;
    let mut right = (ind << 1) | 1;
    let mut left = (ind + 1) << 1;

    if left < end && logger.cmp_lt(arr, len - right - 1, len - left - 1) {
        left = right;
    }

    while left < end && logger.cmp_lt(arr, len - left - 1, len - ind - 1) {
        logger.swap(arr, len - ind - 1, len - left - 1);
        ind = left;
        right = (ind << 1) | 1;
        left = (ind + 1) << 1;
        if left < end && logger.cmp_lt(arr, len - right - 1, len - left - 1) {
            left = right;
        }
    }
}
