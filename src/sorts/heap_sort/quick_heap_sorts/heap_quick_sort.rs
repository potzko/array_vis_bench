use crate::create_sort;

create_sort!(sort, "heap sort", "O(N*log(N))", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    let split: usize = arr.len() / 2;
    let split_len = arr[split..].len();
    first_heapify_lt(&mut arr[0..split], logger);
    first_heapify_gt(&mut arr[split..], logger);
    while logger.cond_swap_lt(arr, split, 0) {
        heapify_lt(arr, 0, split, logger);
        heapify_gt(&mut arr[split..], 0, split_len, logger);
    }
    sort(&mut arr[0..split], logger);
    sort(&mut arr[split..], logger);
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
        heapify_gt(arr, i, len, logger);
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

fn heapify_gt<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
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
