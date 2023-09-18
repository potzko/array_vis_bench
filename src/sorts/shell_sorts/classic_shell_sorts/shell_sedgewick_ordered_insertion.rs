const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^(4/3)))";
const NAME: &str = "shell sort sedgewick jumps";

use crate::traits;
pub struct ShellSort {}

#[inline]
fn sequence_sedgewick(iter: usize) -> usize {
    if iter == 0 {
        return 1;
    }
    4_usize.pow(iter as u32) + 3 * 2_usize.pow(iter as u32 - 1) + 1
}

fn vec_sedgewick(len: usize) -> Vec<usize> {
    let mut ret = Vec::new();
    let mut iter = 0;
    loop {
        let num = sequence_sedgewick(iter);
        if num >= len {
            break;
        }
        ret.push(num);
        iter += 1
    }
    ret
}

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
    for &jump in vec_sedgewick(end - start).iter().rev() {
        logger.log(Mark(format!("jump = {}", jump)));
        for i in 0..jump {
            insertion_sort_jump(arr, i, end, jump, logger);
        }
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
