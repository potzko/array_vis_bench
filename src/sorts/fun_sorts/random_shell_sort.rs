const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2.5)";
const NAME: &str = "shell sort hibbard jumps";
use rand::Rng;

fn random_vec<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    len: usize,
    logger: &mut U,
) -> Vec<usize> {
    let length = (len as f64).sqrt() as usize;
    let mut rnd = rand::thread_rng();
    let mut ret = logger.create_aux_arr(length);
    for i in 0..length - 1 {
        logger.write_data_u(&mut ret, i, rnd.gen_range(0..len));
    }
    logger.write_data_u(&mut ret, length - 1, 1);

    sort_u(&mut ret, logger);
    ret
}
use crate::traits;
pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    let jumps = random_vec(arr.len(), logger);
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

fn sort_u<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [usize], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    let jumps = random_vec(arr.len(), logger);
    for &jump in jumps.iter().rev() {
        logger.mark_mssg(&format!("jump = {}", jump));
        for i in jump..arr.len() {
            let temp = arr[i];
            let mut j = i;
            while j >= jump && logger.cmp_gt_data_u(arr, j - jump, temp) {
                logger.write_u(arr, j, j - jump);
                j -= jump;
            }
            logger.write_data_u(arr, j, temp);
        }
    }
}
