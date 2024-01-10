const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "heap sort";

use crate::traits;
use crate::traits::log_traits::SortLogger;
pub struct HeapSort {}

const HEAP_SIZE: usize = 2;

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

use rand::Rng;

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    first_heapify(arr, logger);

    for i in (1..arr.len()).rev() {
        logger.swap(arr, 0, i);
        heapify(arr, 0, i, logger);
    }
}

fn first_heapify<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
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

fn heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use std::cmp::min;

    let mut ind = start;
    let mut max_child = start * HEAP_SIZE + 1;
    max_child = max_ind(arr, max_child, min(max_child + HEAP_SIZE, end), logger);

    while max_child < end {
        if logger.cond_swap_ge(arr, max_child, ind) {
            ind = max_child;
            max_child = max_ind(
                arr,
                ind * HEAP_SIZE + 1,
                min(ind * HEAP_SIZE + HEAP_SIZE + 1, end),
                logger,
            );
        } else {
            break;
        }
    }
}

fn max_ind<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> usize {
    let mut max = start;
    for i in start + 1..end {
        if logger.cmp_le(arr, max, i) {
            max = i
        }
    }
    max
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
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

fn quick_select<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
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
