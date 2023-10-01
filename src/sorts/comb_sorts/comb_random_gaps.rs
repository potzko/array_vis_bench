const MAX_SIZE: usize = 10000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "comb sort";

use crate::traits;
use rand::Rng;

pub struct CombSort {}

impl traits::sort_traits::SortAlgo for CombSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        sort_helper::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort_helper<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut rng = rand::thread_rng();
    sort(arr, &mut rng, logger)
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],

    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) {
    while comb_iter(arr, 1, logger) {
        comb_iter(arr, rng.gen_range(0..arr.len()), logger);
    }
}

fn comb_iter<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) -> bool {
    if arr.len() < 2 {
        return false;
    }

    let mut swap_flag = false;

    for i in jump..arr.len() {
        if logger.cond_swap_lt(arr, i, i - jump) {
            swap_flag = true;
        }
    }
    swap_flag
}
