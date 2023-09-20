const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shell sort";

use crate::traits;
pub struct ShellSort {}

impl traits::sort_traits::SortAlgo for ShellSort {
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

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    let mut jump = (end - start) / 2;
    while jump >= 1 {
        logger.mark(format!("jump = {}", jump));
        for i in start..jump + start {
            insertion_sort_jump(arr, i, end, jump, logger);
        }
        jump /= 2;
    }
}

fn insertion_sort_jump<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    jump: usize,
    logger: &mut U,
) {
    for i in (start..end).step_by(jump) {
        let mut ind = i;
        while ind != start {
            if logger.cond_swap_le(arr, ind, ind - jump) {
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
