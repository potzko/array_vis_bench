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
use std::marker::PhantomData;

pub struct SortImp<T: Ord + Copy, U: traits::log_traits::SortLogger<T>> {
    _markers: (PhantomData<T>, PhantomData<U>),
}

impl<T: Ord + Copy, U: traits::log_traits::SortLogger<T>> traits::sort_traits::SortAlgo<T, U>
    for SortImp<T, U>
{
    fn max_size() -> usize {
        MAX_SIZE
    }
    fn big_o() -> &'static str {
        BIG_O
    }
    fn sort(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name() -> &'static str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let jumps = vec_knuth(arr.len());
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
