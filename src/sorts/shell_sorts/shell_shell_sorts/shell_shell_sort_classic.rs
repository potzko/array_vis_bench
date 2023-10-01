const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shell sort 2.25 shrink factor";

use crate::traits;
pub struct ShellSort {}

impl traits::sort_traits::SortAlgo for ShellSort {
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
    sort(arr, 1, logger);
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    let len = arr.len() / jump;

    if len == 0 {
        return;
    }
    sort(arr, jump * 2, logger);
    sort(&mut arr[jump..], jump * 2, logger);
    insertion_sort_jump(arr, jump, logger);
}

fn insertion_sort_jump<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    for i in (0..arr.len()).step_by(jump) {
        let mut ind = i;
        while ind != 0 {
            if logger.cond_swap_le(arr, ind, ind - jump) {
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
