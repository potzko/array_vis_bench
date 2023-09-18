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
    use crate::traits::log_traits::SortLog::*;
    let mut jump = (end - start) / 2;
    while jump >= 1 {
        logger.log(Mark(format!("jump = {}", jump)));
        for i in 0..jump {
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
        logger.log(traits::log_traits::SortLog::Mark(format!(
            "inserting value at index {}",
            i
        )));
        let mut ind = i;
        while ind != start {
            logger.log(traits::log_traits::SortLog::Cmp {
                name: &arr as *const _ as usize,
                ind_a: ind,
                ind_b: ind - jump,
                result: arr[ind] < arr[ind - jump],
            });
            if arr[ind] < arr[ind - jump] {
                logger.log(traits::log_traits::SortLog::Swap {
                    name: &arr as *const _ as usize,
                    ind_a: ind,
                    ind_b: ind - jump,
                });
                arr.swap(ind, ind - jump);
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
