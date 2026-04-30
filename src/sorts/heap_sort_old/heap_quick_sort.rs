use crate::create_sort;
use rand::Rng;

create_sort!(sort, "heap sort", "O(N*log(N))", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    first_heapify(arr, logger);

    for i in (1..arr.len()).rev() {
        logger.swap(arr, 0, i);
        heapify(arr, 0, i, logger);
    }
}

fn first_heapify<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    if arr.len() < 2 {
        return;
    }
    let mut rng = rand::thread_rng();
    let mut num = 1;
    while num < arr.len() {
        num <<= 1;
    }
    //arr.select_nth_unstable(num >> 1);
    quick_select(arr, num >> 1, &mut rng, logger);
    while num > 1 {
        num >>= 1;
        //arr[..num].select_nth_unstable(num >> 1);
        quick_select(&mut arr[..num], num >> 1, &mut rng, logger)
    }
}

fn heapify<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
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

fn partition<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) -> usize {
    // Choose a random index between start and end - 1 as the pivot
    let pivot: T = arr[rng.gen_range(0..arr.len())];
    //println!("{}, {}", arr.len(), pivot_index);

    // Swap the pivot with the last element
    let mut small = 0;
    for i in 0..arr.len() {
        if logger.cmp_gt_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    small
}

fn quick_select<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    target: usize,
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) {
    //println!("{}, {}", arr.len(), target);
    let piv = partition(arr, rng, logger);
    if piv == target {
        return;
    }
    if piv < target {
        quick_select(&mut arr[piv..], target - piv, rng, logger);
    } else {
        quick_select(&mut arr[0..piv], target, rng, logger)
    };
}
