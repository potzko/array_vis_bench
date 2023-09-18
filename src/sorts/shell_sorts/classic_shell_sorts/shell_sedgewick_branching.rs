const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^(4/3)))";
const NAME: &str = "shell sort branching sedgewick jumps";

#[inline]
fn sequence_sedgewick_branching(iter: usize) -> usize {
    if iter % 2 == 1 {
        8 * 2_usize.pow(iter as u32) - 6 * 2_usize.pow((iter as u32 + 1) / 2) + 1
    } else {
        9 * (2_usize.pow(iter as u32) - 2_usize.pow(iter as u32 / 2)) + 1
    }
}

fn vec_sedgewick(len: usize) -> Vec<usize> {
    let mut ret = Vec::new();
    let mut iter = 0;
    loop {
        let num = sequence_sedgewick_branching(iter);
        if num >= len {
            break;
        }
        ret.push(num);
        iter += 1
    }
    ret
}

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
    let len = end - start;
    let jumps = vec_sedgewick(len);
    for &jump in jumps.iter().rev() {
        logger.log(Mark(format!("jump = {}", jump)));
        for i in jump..end - start {
            let temp = arr[i];
            let mut j = i;
            while j >= jump {
                logger.log(CmpOuterData {
                    name: &arr as *const _ as usize,
                    ind: j - jump,
                    data: temp,
                    result: arr[j - jump] > temp,
                });
                if arr[j - jump] > temp {
                    logger.log(Write {
                        name: &arr as *const _ as usize,
                        ind_a: j,
                        ind_b: j - jump,
                    });
                    arr[j] = arr[j - jump];
                    j -= jump;
                } else {
                    break;
                }
            }
            logger.log(WriteOutOfArr {
                name: &arr as *const _ as usize,
                ind: j,
                data: temp,
            });
            arr[j] = temp;
        }
    }
}
