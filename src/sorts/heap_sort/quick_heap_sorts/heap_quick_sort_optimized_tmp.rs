const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "heap sort";

use crate::traits;
use crate::traits::*;
pub struct HeapSort {}

impl traits::sort_traits::SortAlgo for HeapSort {
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

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let ins_sort = crate::sorts::insertion_sorts::insertion_sort::InsertionSort {};
    if arr.len() < 16 {
        ins_sort.sort(arr, logger);
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_lt_left(&mut arr[0..split], logger);
    first_heapify_gt_right(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, split, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_left(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split..], logger);
    ins_sort.sort(arr, logger);
}

fn sort_left<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 16 {
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_gt_right(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, split, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_left(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split + 1..], logger);
}

fn sort_right<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 16 {
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_lt_left(&mut arr[0..split], logger);
    first_heapify_gt_right(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, split, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_left(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split + 1..], logger);
}

fn first_heapify_lt_left<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let len = arr.len();
    for i in (0..len / 2).rev() {
        heapify_lt(arr, i, len, logger);
    }
}

fn first_heapify_gt_right<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let len = arr.len();
    for i in (0..len / 2).rev() {
        heapify_gt_left(arr, i, len, logger);
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

fn heapify_gt_left<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut ind = start;
    let mut left_offset = (ind << 1) | 1;
    let mut right_offset = (ind + 1) << 1;

    if left_offset > end
        && logger.cmp_lt(
            arr,
            arr.len() - 1 - right_offset,
            arr.len() - 1 - left_offset,
        )
    {
        left_offset = right_offset;
    }

    while left_offset < end && logger.cmp_lt(arr, arr.len() - 1 - left_offset, arr.len() - 1 - ind)
    {
        logger.swap(arr, arr.len() - 1 - ind, arr.len() - 1 - left_offset);
        ind = left_offset;
        left_offset = (ind << 1) | 1;
        right_offset = (ind + 1) << 1;
        if right_offset < end
            && logger.cmp_lt(
                arr,
                arr.len() - 1 - right_offset,
                arr.len() - 1 - left_offset,
            )
        {
            left_offset = right_offset;
        }
    }
}
