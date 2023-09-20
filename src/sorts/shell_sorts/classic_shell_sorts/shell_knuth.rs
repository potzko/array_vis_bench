const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^1.5)";
const NAME: &str = "shell sort knuth jumps";

#[inline]
fn sequence_knuth(iter: usize) -> usize {
    (3_usize.pow((iter + 1).try_into().unwrap()) - 1) / 2
}

fn vec_knuth(len: usize) -> Vec<usize> {
    let mut ret = Vec::new();
    let mut iter = 0;
    loop {
        let num = sequence_knuth(iter);
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
    let len = end - start;
    let jumps = vec_knuth(len);
    for &jump in jumps.iter().rev() {
        logger.mark_mssg(&format!("jump = {}", jump));
        for i in start + jump..end - start {
            let temp = arr[i];
            let mut j = i;
            while j >= jump + start && logger.cmp_gt_data(arr, j - jump, temp) {
                logger.write(arr, j, j - jump);
                j -= jump;
            }
            logger.write_data(arr, j, temp);
        }
    }
}
