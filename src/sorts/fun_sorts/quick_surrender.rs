const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick surrender";

use crate::traits;
use rand::Rng;
pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) {
        sort::<T, U>(arr, start, end, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn quick_select<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    target: usize,
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) {
    let piv = partition(arr, start, end, rng, logger);
    if piv == target {
        return;
    }
    if piv < target {
        quick_select(arr, piv + 1, end, target, rng, logger);
    } else {
        quick_select(arr, start, piv, target, rng, logger)
    };
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut rng = rand::thread_rng();
    for i in (start..end).step_by(2) {
        quick_select(arr, i, end, i, &mut rng, logger);
    }
    //sort(arr, start, end, &mut rng, logger)
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) -> usize {
    // Choose a random index between start and end - 1 as the pivot
    let pivot_index: usize = rng.gen_range(start..end);
    // Swap the pivot with the last element
    logger.swap(arr, pivot_index, end - 1);

    let pivot = arr[end - 1];
    let mut small = start;
    for i in start..end - 1 {
        if logger.cmp_lt_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, end - 1);
    small
}
