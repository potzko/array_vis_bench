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

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let jumps = vec_sedgewick(arr.len());
    for &jump in jumps.iter().rev() {
        logger.mark_mssg(&format!("jump = {}", jump));
        for i in jump..arr.len() {
            let temp = arr[i];
            let mut j = i;
            while j >= jump && logger.cmp_gt_data(arr, j - jump, temp) {
                logger.write(arr, j, j - jump);
                j -= jump;
            }
            logger.write_data(arr, j, temp);
        }
    }
}
