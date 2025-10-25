use crate::create_sort;
use crate::traits::*;
use sort_registry_macro::SortRegistry;

create_sort!(sort, "heap quick sort optimized", "O(N*log(N))", false);

/// Heap Quick Sort Optimized implementation
#[derive(SortRegistry)]
pub struct HeapQuickSortOptimized;

impl HeapQuickSortOptimized {
    pub fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        if arr.len() < 16 {
            SmallSort::sort(arr, logger);
            return;
        }
        let split: usize = arr.len() / 2;
        let split_len = arr[split..].len();
        first_heapify_lt(&mut arr[0..split], logger);
        first_heapify_gt(&mut arr[split..], logger);
        while logger.cond_swap_lt(arr, split, 0) {
            heapify_lt(arr, 0, split, logger);
            heapify_gt_left(&mut arr[split..], 0, split_len, logger);
        }
        sort_left(&mut arr[0..split], logger);
        sort_right(&mut arr[split..], logger);
        SmallSort::sort(arr, logger);
    }
}

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    HeapQuickSortOptimized::sort(arr, logger);
}

fn sort_left<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    if arr.len() < 16 {
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_gt(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, split, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_left(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split..], logger);
}

fn sort_right<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    if arr.len() < 16 {
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_lt(&mut arr[0..split], logger);
    first_heapify_gt(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, split, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt_left(&mut arr[split..], 0, split_len, logger);
    }
    sort_left(&mut arr[0..split], logger);
    sort_right(&mut arr[split..], logger);
}

fn first_heapify_lt<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let len = arr.len();
    for i in (0..len / 2).rev() {
        heapify_lt(arr, i, len, logger);
    }
}

fn first_heapify_gt<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let len = arr.len();
    for i in (0..len / 2).rev() {
        heapify_gt_left(arr, i, len, logger);
    }
}

fn heapify_lt<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
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

fn heapify_gt_left<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut ind = start;
    let mut left = (ind << 1) | 1;
    let mut right = (ind + 1) << 1;

    if right < end && logger.cmp_lt(arr, right, left) {
        left = right;
    }

    while left < end && logger.cmp_lt(arr, left, ind) {
        logger.swap(arr, ind, left);
        ind = left;
        left = (ind << 1) | 1;
        right = (ind + 1) << 1;
        if right < end && logger.cmp_lt(arr, right, left) {
            left = right;
        }
    }
}
